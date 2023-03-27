use chrono::DateTime;
use clap::Parser;
use helper::uuid;
use helper::uuid::ToBase62;
use kernel::entity::{Account, Authentication};
use kernel::Result;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(long)]
    mail: String,
    #[arg(long)]
    password: String,
}
#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let args = Args::parse();
    let id = uuid::new_v4().to_base62();
    let account = Account::new(id.clone(), id.clone(), id.clone(), DateTime::default());
    let salt = uuid::new_v4().to_base62();
    let authentication = Authentication::new(
        id,
        args.mail,
        salt.clone(),
        helper::auth::stretch_password(&args.password, &salt)?,
    );
    println!("{}", serde_json::to_string_pretty(&account)?);
    println!("{}", serde_json::to_string_pretty(&authentication)?);
    Ok(())
}
