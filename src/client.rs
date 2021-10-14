use tetra::graphics::{self, Color, Texture};
use tetra::math::Vec2;
use tetra::{TetraError};
use tetra::{Context, ContextBuilder, State};
use generated_shared::game_proto_client::GameProtoClient;
use generated_shared::{FloatTuple, PlayGameRequest, PlayGameResponse};

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
        .run(|ctx|GameState::new(ctx, &mut client))
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
    client: GameProtoClient<tonic::transport::Channel>,
}
impl GameState {
        fn new(ctx: &mut Context, client : &mut GameProtoClient<tonic::transport::Channel>) -> tetra::Result<GameState> {
            let player1_texture = Texture::new(ctx, "./resources/player1.png")?;
            let ball_texture = Texture::new(ctx, "./resources/ball.png")?;
            let player2_texture = Texture::new(ctx, "./resources/player2.png")?;
            let play_request = GameState::play_request(&player1_texture, &player2_texture, &ball_texture, client);
            let ball = play_request.ball.expect("Cannot get ball's data from server");
            let ball_position = ball.position.expect("Cannot get ball position from server");
            let ball_position = Vec2::new(
                ball_position.x,
                ball_position.y,
            );
            let ball_velocity = ball.velocity.expect("Cannot get ball velocity from server");
            let ball_velocity = Vec2::new(
                ball_velocity.x,
                ball_velocity.y,
            );
            let player1_position = &play_request.player1_position
                .expect("Cannot get player position from server");
            let player1_position = Vec2::new(
                player1_position.x,
                player1_position.y,
            );
            let player2_position = &play_request.player2_position
                .expect("Cannot get player position from server");
            let player2_position = Vec2::new(
                player2_position.x,
                player2_position.y,
            );
            let player_number = play_request.current_player_number;
            Ok(GameState {
                player1: Entity::new(&player1_texture, player1_position),
                player2: Entity::new(&player2_texture, player2_position),
                ball: Entity::with_velocity(&ball_texture, ball_position, ball_velocity),
                player_number,
                players_count: player_number,
                client: client.clone(),
            })
        }
    #[tokio::main]
    async fn play_request(player1_texture: &Texture, player2_texture: &Texture, ball_texture: &Texture,
                          client : &mut GameProtoClient<tonic::transport::Channel>) -> PlayGameResponse {
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

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        self.player1.texture.draw(ctx, self.player1.position);
        self.player2.texture.draw(ctx, self.player2.position);
        self.ball.texture.draw(ctx, self.ball.position);
        Ok(())
    }
}