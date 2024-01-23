use poise::serenity_prelude as serenity;
use sqlx::{mysql::MySqlPool, Executor, MySql, Pool};
use url::Url;

struct Data {db: Pool<MySql>} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


#[poise::command(slash_command, prefix_command)]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    let configuration = poise::builtins::HelpConfiguration {
        // [configure aspects about the help message here]
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), configuration).await?;
    Ok(())
}


#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();
    let dbuser = std::env::var("SQL_USER").expect("missing SQL_USER");
    let dbpass = std::env::var("SQL_PASS").expect("missing SQL_PASS");
    let dbhost = std::env::var("SQL_HOST").expect("missing SQL_HOST");
    let dbdatabase = std::env::var("SQL_DATABASE").expect("missing SQL_DATABASE");
    let conn_str = format!("mysql://{dbhost}:3306/{dbdatabase}");
    let mut uri = Url::parse(&conn_str).unwrap();
    uri.set_username(&dbuser).unwrap();
    uri.set_password(Some(&dbpass)).unwrap();
    let uri = uri.as_str();
    let pool = MySqlPool::connect(uri).await.unwrap();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![help(),setckey()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {db: pool})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

/// Register your ckey with the ss13 whitelist. Does not need to be formatted in any particular way.
#[poise::command(slash_command)]
pub async fn setckey(
    ctx: Context<'_>,
    #[rest]
    #[description = "BYOND CKey"]
    ckey: String,
) -> Result<(), Error> {
    let userid = ctx.author().id.get();

    let db = &ctx.data().db;

    db.execute(sqlx::query("INSERT INTO whitelist (discord_id, ckey) VALUES (?, ?) AS new ON DUPLICATE KEY UPDATE ckey = new.ckey").bind(userid).bind(&ckey)).await?;

    ctx.say("Successfully created or updated your record!").await?;

    return Ok(());
}