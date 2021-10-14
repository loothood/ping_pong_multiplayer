use tonic::{transport::Server, Response};
use generated_shared::game_proto_server::{GameProto, GameProtoServer};
use generated_shared::{Ball, FloatTuple, PlayGameRequest, PlayGameResponse};
use std::sync::{Mutex, Arc};
use tetra::math::Vec2;
use rand::Rng;

mod generated_shared;

const BALL_SPEED: f32 = 5.0;

#[derive(Clone)]
struct Entity {
    texture_size: Vec2<f32>,
    position: Vec2<f32>,
    velocity: Vec2<f32>,
}
impl Entity {
    fn new(texture_size: Vec2<f32>, position: Vec2<f32>) -> Entity {
        Entity::with_velocity(texture_size, position, Vec2::zero())
    }
    fn with_velocity(texture_size: Vec2<f32>, position: Vec2<f32>, velocity: Vec2<f32>) -> Entity {
        Entity { texture_size, position, velocity }
    }
}
#[derive(Clone)]
struct World {
    player1: Entity,
    player2: Entity,
    ball: Entity,
    world_size: Vec2<f32>,
    winner: u32,
}
pub struct PlayGame {
    world: Arc<Mutex<Option<World>>>,
    players_count: Arc<Mutex<u32>>,
}
impl PlayGame {
    fn new() -> PlayGame {
        PlayGame {
            world: Arc::new(Mutex::new(None)),
            players_count: Arc::new(Mutex::new(0u32)),
        }
    }
    fn init(&self, window_size: FloatTuple, player1_texture: FloatTuple,
            player2_texture: FloatTuple, ball_texture: FloatTuple) {
        let window_width = window_size.x;
        let window_height = window_size.y;
        let world = Arc::clone(&self.world);
        let mut world = world.lock().unwrap();
        let players_count = Arc::clone(&self.players_count);
        let players_count = players_count.lock().unwrap().clone();
        let mut ball_velocity = 0f32;
        if players_count >= 2 {
            let num = rand::thread_rng().gen_range(0..2);
            if num == 0 {
                ball_velocity = -BALL_SPEED;
            } else {
                ball_velocity = BALL_SPEED;
            }
        }
        *world =
            Option::Some(World {
                player1: Entity::new(
                    Vec2::new(player1_texture.x, player1_texture.y),
                    Vec2::new(
                        16.0,
                        (window_height - player1_texture.y) / 2.0,
                    ),
                ),
                player2: Entity::new(
                    Vec2::new(player2_texture.x, player2_texture.y),
                    Vec2::new(
                        window_width - player2_texture.y - 16.0,
                        (window_height - player2_texture.y) / 2.0,
                    ),
                ),
                ball: Entity::with_velocity(
                    Vec2::new(ball_texture.x, ball_texture.y),
                    Vec2::new(
                        window_width / 2.0 - ball_texture.x / 2.0,
                        window_height / 2.0 - ball_texture.y / 2.0,
                    ),
                    Vec2::new(
                        ball_velocity,
                        0f32,
                    ),
                ),
                world_size: Vec2::new(window_size.x, window_size.y),
                // No one win yet
                winner: 2,
            });
    }
    fn increase_players_count(&self) {
        let players_count = Arc::clone(&self.players_count);
        let mut players_count = players_count.lock().unwrap();
        *players_count += 1;
    }
}
#[tonic::async_trait]
impl GameProto for PlayGame {
    async fn play_request(
        &self,
        request: tonic::Request<PlayGameRequest>,
    ) -> Result<tonic::Response<PlayGameResponse>, tonic::Status> {
        let pgr: PlayGameRequest = request.into_inner();
        let window_size = pgr.window_size.unwrap();
        let player1_texture = pgr.player1_texture.unwrap();
        let player2_texture = pgr.player2_texture.unwrap();
        let ball_texture_height = pgr.ball_texture.unwrap();
        self.increase_players_count();
        self.init(window_size, player1_texture,
                  player2_texture, ball_texture_height);
        let world = Arc::clone(&self.world).lock().unwrap().as_ref().unwrap().clone();
        let current_players = Arc::clone(&self.players_count);
        let current_players = current_players.lock().unwrap();
        let reply = PlayGameResponse {
            player1_position: Option::Some(FloatTuple {
                x: world.player1.position.x,
                y: world.player1.position.y,
            }),
            player2_position: Option::Some(FloatTuple {
                x: world.player2.position.x,
                y: world.player2.position.y,
            }),
            current_player_number: current_players.clone(),
            players_count: current_players.clone(),
            ball: Option::Some(Ball {
                position: Option::Some(FloatTuple {
                    x: world.ball.position.x,
                    y: world.ball.position.y,
                }),
                velocity: Option::Some(FloatTuple {
                    x: world.ball.velocity.x,
                    y: world.ball.velocity.y,
                }),
            }),
        };
        Ok(Response::new(reply))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let play_game = PlayGame::new();
    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(GameProtoServer::new(play_game))
        .serve(addr)
        .await?;
    Ok(())
}