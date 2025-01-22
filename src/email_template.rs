use crate::util::read_env_or_panic;
use uuid::Uuid;

fn unsubscribe_link(subscriber_id: Uuid) -> String {
    let app_base_url = read_env_or_panic("APP_BASE_URL");
    format!(
        "{}/subscriptions/unsubscribe?id={}",
        app_base_url, subscriber_id
    )
}

pub fn html_version(subscriber_id: Uuid, html_content: &str) -> String {
    let link = unsubscribe_link(subscriber_id);
    format!(
        r#"
        <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto;">
            {html_content}
    
            <hr style="border: none; border-top: 1px solid #ddd; margin: 20px 0;"/>
    
            <footer style="font-size: 12px; color: #666;">
                <p>
                    You're receiving this because you subscribed to Joe Hasson's Blog.<br/>
                    <a href="{link}" style="color: #666;">Unsubscribe</a>
                </p>
            </footer>
        </div>"#
    )
}

pub fn text_version(subscriber_id: Uuid, text_content: &str) -> String {
    let link = unsubscribe_link(subscriber_id);
    format!(
        "{text_content}\n\n\
        -------------------------------------------\n\
        You're receiving this because you subscribed to Joe Hasson's Blog.\n\
        Unsubscribe: {link}\n"
    )
}
