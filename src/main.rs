use std::{
    fs::File,
    io::Read,
};

use code_time_monitor::*;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let mut config: String = String::new();
    File::open("./config.txt").unwrap().read_to_string(&mut config).unwrap();
    let uri = config.lines().collect::<Vec<&str>>()[0];

    let collection = match try_open_client(uri).await {
        Err(e) => return e,
        Ok(v) => v
    };

    match try_add_from_file(&format!("{}/.time", env!("HOME")), &collection).await {
        Err(_) => println!("Error: impossible to add time from file"),
        _ => ()
    };

    println!("Total time: {}", millis_to_readable(sum_time(&collection).await?));

    println!("Latest 24h: {}", millis_to_readable(latest(&collection, 24*3600).await?));

    println!("Daily average: {}", millis_to_readable(daily_average(&collection).await?.round() as i64));

    Ok(())
}
