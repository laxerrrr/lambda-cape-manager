use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use mojang::Player;

use uuid::Uuid;
use serde::{Serialize, Deserialize};

use curl::easy::Easy;

struct Handler;

pub type LambdaUserList = Vec<LambdaUser>;

#[derive(Serialize, Deserialize, Clone)]
pub struct LambdaUser {
    id: f64,
    capes: Vec<Cape>,
    is_premium: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cape {
    cape_uuid: String,
    player_uuid: String,
    #[serde(rename = "type")]
    cape_type: Type,
    color: Color,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Color {
    primary: String,
    border: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Type {
    #[serde(rename = "CONTRIBUTOR")]
    Contributor,
}

pub struct PlayerEmbedData {
    username : String,
    uuid : String,
    skin_url : String,
    capes : Vec<Cape>
}

impl PlayerEmbedData {
    fn new(lambda_user : LambdaUser) -> PlayerEmbedData {

        let capes : Vec<Cape> = lambda_user.capes;

        let uuid : String = capes[0].player_uuid.clone();

        let player = Player::new(&uuid).unwrap().add_skin().unwrap(); //Creating player object with Mojang API

        let username = player.name;
        let skin_url = player.skin_url.unwrap();

        PlayerEmbedData {username, uuid, skin_url, capes}
    }
}


#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!test" {
            
            let lambda_user_list : LambdaUserList = getFullCapeData();
            
            for user in lambda_user_list {

                let embed_data : PlayerEmbedData = PlayerEmbedData::new(user.clone());

                if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| m
                        .embed(|e| e
                            .title(embed_data.username)
                            .image(embed_data.skin_url)

                        )).await {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn getFullCapeData() -> LambdaUserList {
    let mut html: String = String::new();
    {
        let mut easy = Easy::new();
        easy.url("https://raw.githubusercontent.com/lambda-client/cape-api/capes/capes.json").unwrap();
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            html.push_str(std::str::from_utf8(data).unwrap());
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    let res : LambdaUserList = serde_json::from_str(&html).unwrap();
    return res;
 }

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client =
        Client::builder(&token).event_handler(Handler).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

