use clap::Parser;
use helper::env::get_var;
use helper::time::current_time;
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
    let account = Account::new(id.clone(), id.clone(), id.clone(), current_time());
    let salt = uuid::new_v4().to_base62();
    let pepper = get_var::<String>("AUTH_PEPPER").unwrap();
    let count = get_var::<i64>("AUTH_STRETCH_COUNT").unwrap();
    let authentication = Authentication::new(
        id,
        args.mail,
        salt.clone(),
        helper::auth::stretch_password(&args.password, &salt, &pepper, count)?,
    );
    println!("{}", serde_json::to_string_pretty(&account)?);
    println!("{}", serde_json::to_string_pretty(&authentication)?);
    Ok(())
}
