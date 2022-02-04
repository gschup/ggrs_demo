mod ex_game;

use async_executor::LocalExecutor;
use ex_game::{GGRSConfig, Game};
use ggrs::{GGRSError, GGRSEvent, PlayerType, SessionBuilder, SessionState};
use instant::{Duration, Instant};
use macroquad::prelude::*;
use matchbox_socket::WebRtcNonBlockingSocket;

use crate::ex_game::ConnectionStatus;

const NUM_PLAYERS: usize = 2;
const MATCHBOX_ADDR: &str = "ws://127.0.0.1:3536";
const FPS: f64 = 60.0;

#[macroquad::main("FightingBase")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create a matchbox socket
    info!("Constructing socket...");
    let room_url = format!("{MATCHBOX_ADDR}/next_{NUM_PLAYERS}");
    let (mut socket, message_loop) = WebRtcNonBlockingSocket::new(room_url);
    let local_executor = LocalExecutor::new();
    let task = local_executor.spawn(message_loop);
    task.detach();

    // wait until other player is there
    info!("Waiting for other player...");
    while socket.connected_peers().len() < NUM_PLAYERS - 1 {
        local_executor.try_tick();
        socket.accept_new_connections();
        next_frame().await;
    }

    // Create a new game
    info!("Starting game...");
    let mut game = Game::new(NUM_PLAYERS);

    // create a GGRS session
    info!("Building GGRS Session...");
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(NUM_PLAYERS)
        .with_fps(FPS as usize)? // (optional) set expected update frequency
        .with_input_delay(2); // (optional) set input delay for the local player

    // add players
    for (i, player_type) in socket.players().iter().enumerate() {
        sess_build = sess_build.add_player(player_type.clone(), i)?;

        if matches!(player_type, PlayerType::Local) {
            game.set_connection_status(vec![i], ConnectionStatus::Local);
        }
    }

    // start the GGRS session
    let mut sess = sess_build.start_p2p_session(socket)?;

    // time variables for tick rate
    let mut last_update = Instant::now();
    let mut accumulator = Duration::ZERO;

    loop {
        // communicate, receive and send packets
        local_executor.try_tick();
        sess.poll_remote_clients();
        local_executor.try_tick();

        // handle GGRS events
        let events: Vec<GGRSEvent<GGRSConfig>> = sess.events().collect();
        for event in events {
            info!("Event: {:?}", event);
            match event {
                GGRSEvent::Synchronized { addr } => game.set_connection_status(
                    sess.handles_by_address(addr),
                    ConnectionStatus::Running,
                ),
                GGRSEvent::Disconnected { addr } => game.set_connection_status(
                    sess.handles_by_address(addr),
                    ConnectionStatus::Disconnected,
                ),
                GGRSEvent::NetworkInterrupted {
                    addr,
                    disconnect_timeout: _,
                } => game.set_connection_status(
                    sess.handles_by_address(addr),
                    ConnectionStatus::Interrupted,
                ),
                GGRSEvent::NetworkResumed { addr } => game.set_connection_status(
                    sess.handles_by_address(addr),
                    ConnectionStatus::Running,
                ),
                _ => (),
            };
        }

        // get network stats
        for handle in sess.remote_player_handles() {
            game.connection_info[handle].stats = sess.network_stats(handle).ok();
        }

        // frames are only happening if the sessions are synchronized
        if sess.current_state() == SessionState::Running {
            // this is to keep ticks between clients synchronized.
            // if a client is ahead, it will run frames slightly slower to allow catching up
            let fps_delta = if sess.frames_ahead() > 0 {
                (1. / FPS) * 1.1
            } else {
                1. / FPS
            };

            // get delta time from last iteration and accumulate it
            let delta = Instant::now().duration_since(last_update);
            accumulator = accumulator.saturating_add(delta);
            last_update = Instant::now();

            // if enough time is accumulated, we run a frame
            while accumulator.as_secs_f64() > fps_delta {
                // decrease accumulator
                accumulator = accumulator.saturating_sub(Duration::from_secs_f64(fps_delta));

                // add input for all local players
                for handle in sess.local_player_handles() {
                    sess.add_local_input(handle, game.local_input(0))?; // we always call game.local_input(0) in order to get WASD inputs.
                }

                match sess.advance_frame() {
                    Ok(requests) => game.handle_requests(requests),
                    Err(GGRSError::PredictionThreshold) => {
                        info!("Frame {} skipped", sess.current_frame())
                    }
                    Err(e) => return Err(Box::new(e)),
                }
            }
        }

        game.render();
        local_executor.try_tick();
        next_frame().await;
    }
}
