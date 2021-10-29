use tetra::graphics::{self, Color, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::{TetraError};
use tetra::{Context, ContextBuilder, State};
use generated_shared::game_proto_client::GameProtoClient;
use generated_shared::{PlayGameRequest, PlayGameResponse, FloatTuple, WorldStatus, ClientActions};
use tetra::graphics::text::{Text, Font};

mod generated_shared;

const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 720.0;

async fn establish_connection() -> GameProtoClient<tonic::transport::Channel> {
    GameProtoClient::connect("http://[::1]:50051").await.expect("Can't connect to the server")
}

fn main() -> Result<(), TetraError> {
    let rt = tokio::runtime::Runtime::new().expect("Error runtime creation");
    let mut client = rt.block_on(establish_connection());

    ContextBuilder::new("Pong", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .quit_on_escape(true)
        .build()?
        .run(|ctx| GameState::new(ctx, &mut client))
}

impl From<FloatTuple> for Vec2<f32> {
    fn from(data: FloatTuple) -> Self {
        Vec2 {
            x: data.x,
            y: data.y,
        }
    }
}

struct Entity {
    texture: Texture,
    position: Vec2<f32>,
    velocity: Vec2<f32>,
}

impl Entity {
    fn new(texture: &Texture, position: Vec2<f32>) -> Entity {
        Entity::with_velocity(&texture, position, Vec2::zero())
    }

    fn with_velocity(texture: &Texture, position: Vec2<f32>, velocity: Vec2<f32>) -> Entity {
        Entity { texture: texture.clone(), position, velocity }
    }
}

struct GameState {
    player1: Entity,
    player2: Entity,
    ball: Entity,
    player_number: u32,
    players_count: u32,
    winner: u32,
    client: GameProtoClient<tonic::transport::Channel>,
}

impl GameState {
    fn new(ctx: &mut Context, client: &mut GameProtoClient<tonic::transport::Channel>)
           -> tetra::Result<GameState> {
        let player1_texture = Texture::new(ctx, "./resources/player1.png")?;
        let ball_texture = Texture::new(ctx, "./resources/ball.png")?;
        let player2_texture = Texture::new(ctx, "./resources/player2.png")?;
        let play_request =
            GameState::play_request(&player1_texture, &player2_texture, &ball_texture, client);

        let ball = play_request.ball.unwrap();
        let ball_position = Vec2::from(ball.position.unwrap());
        let ball_velocity = Vec2::from(ball.velocity.unwrap());

        let player1_position: FloatTuple = play_request.player1_position
            .expect("Cannot get player position from server");
        let player1_position = Vec2::from(player1_position);

        let player2_position: FloatTuple = play_request.player2_position
            .expect("Cannot get player position from server");
        let player2_position = Vec2::from(player2_position);

        let player_number = play_request.current_player_number;

        Ok(GameState {
            player1: Entity::new(&player1_texture, player1_position),
            player2: Entity::new(&player2_texture, player2_position),
            ball: Entity::with_velocity(&ball_texture, ball_position, ball_velocity),
            player_number,
            players_count: player_number,
            // No winner by default
            winner: 2,
            client: client.clone(),
        })
    }

    fn set_updated_values(&mut self, response: WorldStatus) {
        let players_count = response.players_count;
        self.players_count = players_count;
        if players_count >= 2 {
            let ball = response.ball.unwrap();
            let ball_position = Vec2::from(ball.position.unwrap());
            let ball_velocity = Vec2::from(ball.velocity.unwrap());

            let player1_position: FloatTuple = response.player1_position
                .expect("Cannot get player position from server");
            let player1_position = Vec2::from(player1_position);

            let player2_position: FloatTuple = response.player2_position
                .expect("Cannot get player position from server");
            let player2_position = Vec2::from(player2_position);

            let winner = &response.winner;

            self.winner = winner.clone() as u32;
            self.ball.position = ball_position;
            self.ball.velocity = ball_velocity;
            self.player1.position = player1_position;
            self.player2.position = player2_position;
        }
    }

    #[tokio::main]
    async fn world_update_request(&self, clicked_button_number: u32, player_number: u32) -> WorldStatus {
        let request = tonic::Request::new(ClientActions {
            player_number,
            clicked_button: clicked_button_number,
        });
        let mut client = self.client.clone();
        client.world_update_request(request)
            .await.expect("Cannot get World Update from the server").into_inner()
    }

    #[tokio::main]
    async fn play_request(player1_texture: &Texture, player2_texture: &Texture, ball_texture: &Texture,
                          client: &mut GameProtoClient<tonic::transport::Channel>) -> PlayGameResponse {
        let request = tonic::Request::new(PlayGameRequest {
            window_size: Some(FloatTuple { x: WINDOW_WIDTH, y: WINDOW_HEIGHT }),
            player1_texture: Some(
                FloatTuple { x: player1_texture.width() as f32, y: player1_texture.height() as f32 }
            ),
            player2_texture: Some(
                FloatTuple { x: player2_texture.width() as f32, y: player2_texture.height() as f32 }
            ),
            ball_texture: Some(
                FloatTuple { x: ball_texture.width() as f32, y: ball_texture.height() as f32 }
            ),
        });
        client.play_request(request).await.expect("Cannot get Play Response the server").into_inner()
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        let mut clicked_button = 2;

        if input::is_key_down(ctx, Key::Up) {
            clicked_button = 0;
        }
        if input::is_key_down(ctx, Key::Down) {
            clicked_button = 1;
        }

        let world_update_request =
            self.world_update_request(clicked_button, self.player_number);
        self.set_updated_values(world_update_request);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        // 0 - Player 1 won
        // 1 - Player 2 won
        if self.winner == 2 {
            self.player1.texture.draw(ctx, self.player1.position);
            self.ball.texture.draw(ctx, self.ball.position);
            self.player2.texture.draw(ctx, self.player2.position);
        } else {
            let text_offset: Vec2<f32> = Vec2::new(16.0, 16.0);
            let mut message = format!("Winner is: Player ");
            if self.winner == 0 {
                message += "1";
            } else {
                message += "2";
            }
            let mut t: Text = Text::new(message,
                                        Font::vector(ctx, "./resources/DejaVuSansMono.ttf",
                                                     16.0)?,
            );
            t.draw(ctx, text_offset);
        }
        Ok(())
    }
}