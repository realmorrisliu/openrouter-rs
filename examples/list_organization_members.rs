use openrouter_rs::{
    OpenRouterClient, api::end_users::ListEndUsersParams, types::PaginationOptions,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let management_key =
        std::env::var("OPENROUTER_MANAGEMENT_KEY").expect("OPENROUTER_MANAGEMENT_KEY must be set");

    let client = OpenRouterClient::builder()
        .management_key(management_key)
        .build()?;

    let members = client
        .management()
        .list_organization_members(Some(PaginationOptions::with_offset_and_limit(0, 25)))
        .await?;

    println!("member count: {}", members.total_count);
    for member in members.data {
        println!(
            "{} {} <{}>",
            member.first_name.unwrap_or_default(),
            member.last_name.unwrap_or_default(),
            member.email
        );
    }

    let settings = client.management().get_organization_settings().await?;
    println!(
        "filtered catalog enabled: {}",
        settings.is_filtered_model_catalog_enabled
    );
    let end_users = client
        .management()
        .list_end_users(
            &ListEndUsersParams::builder()
                .limit(25)
                .include_inactive(true)
                .build()?,
        )
        .await?;
    println!("registered end users: {}", end_users.total_count);

    Ok(())
}
