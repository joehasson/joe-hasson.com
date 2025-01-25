use shared::util::read_env_or_panic;
use sqlx::{
    postgres::{PgConnectOptions, PgConnection, PgSslMode},
    Connection, Executor,
};
use std::process::{exit, Command};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    eprintln!("Scanning for new blog posts...");
    let output = Command::new("git")
        .args([
            "diff",
            "--diff-filter=A",
            "--name-only",
            "HEAD^1",
            "HEAD",
            "blog",
        ])
        .output()?
        .stdout;
    let output = String::from_utf8(output).unwrap();

    let mut new_files: Vec<_> = output.split('\n').collect();
    new_files.pop();
    let new_files = new_files;

    if new_files.is_empty() {
        eprintln!("No new blog posts detected.");
        exit(0);
    }

    eprintln!("New files: {:?}", new_files);

    let options = PgConnectOptions::new()
        .host(&read_env_or_panic("DB_HOST"))
        .username(&read_env_or_panic("DB_USER"))
        .password(&read_env_or_panic("DB_PASSWORD"))
        .database(&read_env_or_panic("DB_NAME"))
        .port(
            read_env_or_panic("DB_PORT")
                .parse::<u16>()
                .expect("DB_PORT was not a u16"),
        )
        .ssl_mode(PgSslMode::Prefer);

    let mut conn = PgConnection::connect_with(&options).await?;
    let mut transaction = conn.begin().await?;
    let app_base_url = read_env_or_panic("APP_BASE_URL");

    for file in new_files {
        let slug = file
            .strip_prefix("blog/")
            .unwrap_or_else(|| panic!("Filepath {} does not begin with blog/", file))
            .strip_suffix(".html")
            .unwrap_or_else(|| {
                panic!("Problem {}: Blog entries should have .html extension", file)
            });
        let link = format!("{}/blog/{}", app_base_url, slug);

        let subject = "New blog post";
        let email_html = format!(
            "<p>New blog post! Click <a href={}>here</a> to view.p>",
            link
        );
        let email_text = format!("New blog post! Available at {}.", link);
        let query = sqlx::query!(
            r#"
            INSERT INTO email_delivery_queue
            SELECT gen_random_uuid(), id, $1, $2, $3
            FROM subscriptions
            WHERE confirmed = true
            AND $4 NOT IN (SELECT slug FROM blog_posts)
            "#,
            subject,
            email_html,
            email_text,
            slug
        );
        transaction.execute(query).await.unwrap_or_else(|_| {
            panic!(
                "Failed to enqueue email notifications for new post {}",
                &file
            )
        });

        let query = sqlx::query!(r#" INSERT INTO blog_posts (slug) VALUES ($1)"#, slug);
        transaction
            .execute(query)
            .await
            .unwrap_or_else(|_| panic!("Failed to register new post {} in blog_posts table", slug));
    }

    transaction.commit().await?;

    Ok(())
}
