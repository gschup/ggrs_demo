mod ex_game;
mod lobby;

use async_executor::LocalExecutor;
use ex_game::{FrameStatus, GGRSConfig, Game};
use ggrs::{GgrsError, P2PSession, PlayerType, SessionBuilder, SessionState};
use instant::{Duration, Instant};
use macroquad::prelude::*;
use matchbox_socket::WebRtcSocket;

use crate::ex_game::ConnectionStatus;
use crate::lobby::Lobby;

const NUM_PLAYERS: usize = 2;
const MATCHBOX_ADDR: &str = "ws://127.0.0.1:3536";
const FPS: f64 = 60.0;

enum DemoState {
    Lobby,
    Connecting,
    Game,
}

struct GGRSDemo<'a> {
    state: DemoState,
    executor: LocalExecutor<'a>,
    socket: Option<WebRtcSocket>,
    session: Option<P2PSession<GGRSConfig>>,
    lobby: Lobby,
    game: Game,
    last_update: Instant,
    accumulator: Duration,
}

impl<'a> GGRSDemo<'a> {
    fn new(logo: Texture2D) -> Self {
        Self {
            state: DemoState::Lobby,
            executor: LocalExecutor::new(),
            socket: None,
            session: None,
            game: Game::new(NUM_PLAYERS),
            lobby: Lobby::new(logo),
            last_update: Instant::now(),
            accumulator: Duration::ZERO,
        }
    }

    async fn run(&mut self) {
        loop {
            clear_background(BLACK);
            match &mut self.state {
                DemoState::Lobby => self.run_lobby(),
                DemoState::Connecting => self.run_connecting(),
                DemoState::Game => self.run_game(),
            }
            next_frame().await;
        }
    }

    fn run_lobby(&mut self) {
        if let Some(room_id) = self.lobby.run() {
            info!("Constructing socket...");
            let room_url = format!("{MATCHBOX_ADDR}/{room_id}");
            let (socket, message_loop) = WebRtcSocket::new_ggrs(room_url);
            self.socket = Some(socket);
            let task = self.executor.spawn(message_loop);
            task.detach();
            self.state = DemoState::Connecting;
        }
    }

    fn run_connecting(&mut self) {
        let socket = self
            .socket
            .as_mut()
            .expect("Should only be in connecting state if there exists a socket.");

        self.executor.try_tick();

        // Update peers and count connected ones
        let _peer_updates = socket.update_peers();
        let connected_peers_count = socket.connected_peers().count();

        let info_str = format!(
            "Waiting for {} more player(s)...",
            NUM_PLAYERS - 1 - connected_peers_count
        );
        draw_text(&info_str, 20.0, 20.0, 30.0, WHITE);

        // if we have enough players - we assume there to be only one local player
        if connected_peers_count >= NUM_PLAYERS - 1 {
            // create a new game
            info!("Starting new game...");
            self.game = Game::new(NUM_PLAYERS);
            self.state = DemoState::Game;

            // create a new ggrs session
            let mut sess_build = SessionBuilder::<GGRSConfig>::new()
                .with_num_players(NUM_PLAYERS)
                .with_max_prediction_window(12)
                .expect("Invalid prediction window")
                .with_fps(FPS as usize)
                .expect("Invalid FPS")
                .with_input_delay(2);

            // add players
            for (i, player_type) in socket.players().iter().enumerate() {
                let ggrs_player_type = match player_type {
                    ggrs::PlayerType::Local => PlayerType::Local,
                    ggrs::PlayerType::Remote(peer_id) => PlayerType::Remote(*peer_id),
                    ggrs::PlayerType::Spectator(peer_id) => PlayerType::Spectator(*peer_id),
                };
                sess_build = sess_build
                    .add_player(ggrs_player_type, i)
                    .expect("Invalid player added.");
                if matches!(player_type, ggrs::PlayerType::Local) {
                    self.game
                        .set_connection_status(vec![i], ConnectionStatus::Local);
                }
            }

            // start the GGRS session
            let sess = sess_build
                .start_p2p_session(self.socket.take().unwrap())
                .expect("Session could not be created.");
            self.session = Some(sess);

            // reset time variables for frame ticks
            self.last_update = Instant::now();
            self.accumulator = Duration::ZERO;
        }

        // user can abort
        if is_key_pressed(KeyCode::Escape) {
            self.state = DemoState::Lobby;
            self.socket = None;
            self.executor = LocalExecutor::new();
        }
    }

    fn run_game(&mut self) {
        let sess = self
            .session
            .as_mut()
            .expect("Should only be in game state if there exists a session.");

        // communicate, receive and send packets
        self.executor.try_tick();
        sess.poll_remote_clients();
        self.executor.try_tick();

        // handle GGRS events
        self.game.handle_events(sess);

        // update network stats
        for handle in sess.remote_player_handles() {
            self.game.connection_info[handle].stats = sess.network_stats(handle).ok();
        }

        // this is to keep ticks between clients synchronized.
        // if a client is ahead, it will run frames slightly slower to allow catching up
        let mut fps_delta = 1. / FPS;
        if sess.frames_ahead() > 0 {
            fps_delta *= 1.1;
        }

        // get delta time from last iteration and accumulate it
        let delta = Instant::now().duration_since(self.last_update);
        self.accumulator = self.accumulator.saturating_add(delta);
        self.last_update = Instant::now();

        // if enough time is accumulated, we run a frame
        while self.accumulator.as_secs_f64() > fps_delta {
            // decrease accumulator
            self.accumulator = self
                .accumulator
                .saturating_sub(Duration::from_secs_f64(fps_delta));

            // frames are only happening if the sessions are synchronized
            if sess.current_state() == SessionState::Running {
                // add input for all local players
                for handle in sess.local_player_handles() {
                    sess.add_local_input(handle, self.game.local_input(0))
                        .expect("Invalid player handle"); // we always call game.local_input(0) in order to get WASD inputs.
                }

                match sess.advance_frame() {
                    Ok(requests) => {
                        self.game.handle_requests(requests);
                        self.game.frame_info = if sess.frames_ahead() > 0 {
                            FrameStatus::Slow
                        } else {
                            FrameStatus::Normal
                        }
                    }
                    Err(GgrsError::PredictionThreshold) => self.game.frame_info = FrameStatus::Halt,
                    Err(e) => panic!(
                        "Unknown error happened during P2PSession::<_>::advance_frame(): {e}"
                    ),
                }
            }
        }

        self.game.render();
        self.executor.try_tick();
    }
}

#[macroquad::main("GGRS Demo")]
async fn main() {
    let logo: Texture2D = load_texture("ggrs_logo.png").await.unwrap();
    GGRSDemo::new(logo).run().await;
}
