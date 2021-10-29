use tonic::{transport::Server, Response};
use generated_shared::game_proto_server::{GameProto, GameProtoServer};
use generated_shared::{Ball, ClientActions, FloatTuple, PlayGameRequest, PlayGameResponse, WorldStatus};
use std::sync::{Mutex, Arc};
use tetra::math::Vec2;
use tetra::graphics::Rectangle;
use rand::Rng;

mod generated_shared;

const BALL_SPEED: f32 = 5.0;
const PADDLE_SPEED: f32 = 8.0;
const PADDLE_SPIN: f32 = 4.0;
const BALL_ACC: f32 = 0.05;

impl From<Vec2<f32>> for FloatTuple  {
    fn from(data: Vec2<f32>) -> Self {
        FloatTuple {
            x: data.x,
            y: data.y,
        }
    }
}

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

    fn width(&self) -> f32 {
        self.texture_size.x
    }

    fn height(&self) -> f32 {
        self.texture_size.y
    }

    fn centre(&self) -> Vec2<f32> {
        Vec2::new(
            self.position.x + (self.width() / 2.0),
            self.position.y + (self.height() / 2.0),
        )
    }

    fn bounds(&self) -> Rectangle {
        Rectangle::new(
            self.position.x,
            self.position.y,
            self.width(),
            self.height(),
        )
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

        let ball_velocity = {
            if players_count < 2 {
                0f32
            } else {
                if rand::thread_rng().gen_range(0..2) == 0 {
                    -BALL_SPEED
                } else {
                    BALL_SPEED
                }
            }
        };

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
                // Noone win yet
                winner: 2,
            });
    }

    fn increase_players_count(&self) {
        let players_count = Arc::clone(&self.players_count);
        let mut players_count = players_count.lock().unwrap();
        *players_count += 1;
    }

    fn apply_new_world(&self, new_world: &World) {
        let world = Arc::clone(&self.world);
        let mut world = world.lock().unwrap();
        *world = Option::from(new_world.clone());
    }

    fn update_world(world: &mut World, clicked_button: u32, player_number: u32) {
        // 0 - UP
        // 1 - Down
        if clicked_button == 0 {
            if player_number == 1 {
                world.player1.position.y -= PADDLE_SPEED;
            } else if player_number == 2 {
                world.player2.position.y -= PADDLE_SPEED;
            }
        } else if clicked_button == 1 {
            if player_number == 1 {
                world.player1.position.y += PADDLE_SPEED;
            } else if player_number == 2 {
                world.player2.position.y += PADDLE_SPEED;
            }
        }
        world.ball.position += world.ball.velocity;

        let player1_bounds = world.player1.bounds();
        let player2_bounds = world.player2.bounds();
        let ball_bounds = world.ball.bounds();

        let paddle_hit = if ball_bounds.intersects(&player1_bounds) {
            Some(&world.player1)
        } else if ball_bounds.intersects(&player2_bounds) {
            Some(&world.player2)
        } else {
            None
        };

        if let Some(paddle) = paddle_hit {
            world.ball.velocity.x =
                -(world.ball.velocity.x + (BALL_ACC * world.ball.velocity.x.signum()));

            let offset = (paddle.centre().y - world.ball.centre().y) / paddle.height();

            world.ball.velocity.y += PADDLE_SPIN * -offset;
        }

        if world.ball.position.y <= 0.0
            || world.ball.position.y + world.ball.height() >= world.world_size.y
        {
            world.ball.velocity.y = -world.ball.velocity.y;
        }

        if world.ball.position.x < 0.0 {
            //Player 2 win
            world.winner = 1;
        }

        if world.ball.position.x > world.world_size.x {
            // Player 1 win
            world.winner = 0;
        }
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
            player1_position: Some(FloatTuple::from(world.player1.position)),
            player2_position: Some(FloatTuple::from(world.player2.position)),
            current_player_number: current_players.clone(),
            players_count: current_players.clone(),
            ball: Some(Ball {
                position: Some(FloatTuple::from(world.ball.position)),
                velocity: Some(FloatTuple::from(world.ball.velocity)),
            }),
        };
        Ok(Response::new(reply))
    }

    async fn world_update_request(
        &self,
        request: tonic::Request<ClientActions>,
    ) -> Result<tonic::Response<WorldStatus>, tonic::Status> {
        let client_actions: ClientActions = request.into_inner();
        let clicked_button = client_actions.clicked_button;
        let player_number = client_actions.player_number;

        let players_count = Arc::clone(&self.players_count);
        let players_count = players_count.lock().unwrap().clone();

        let mut world = Arc::clone(&self.world).lock().unwrap().as_ref().unwrap().clone();
        if players_count >= 2 {
            PlayGame::update_world(&mut world, clicked_button, player_number);
        }
        self.apply_new_world(&world);

        let players_count = Arc::clone(&self.players_count);
        let players_count = players_count.lock().unwrap();

        let reply = WorldStatus {
            player1_position: Some(FloatTuple::from(world.player1.position)),
            player2_position: Some(FloatTuple::from(world.player2.position)),
            ball: Some(Ball {
                position: Some(FloatTuple::from(world.ball.position)),
                velocity: Some(FloatTuple::from(world.ball.velocity)),
            }),
            players_count: players_count.clone(),
            winner: world.winner,
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