use mqtt_async_client::{
    client::{Client, Publish, QoS, Subscribe, SubscribeTopic, Unsubscribe, UnsubscribeTopic},
    Error, Result,
};

use tokio::{self, time::Duration};

pub fn plain_client(host: &str, port: u16, username: &str, password: &str) -> Result<Client> {
    Client::builder()
        .set_host(host.to_owned())
        .set_port(port)
        .set_username(Some(username.to_owned()))
        .set_password(Some(password.as_bytes().to_vec()))
        .set_connect_retry_delay(Duration::from_secs(1))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    // for integration tests only. DO NOT COMMIT real values!
    const HOST: &str = "m24.cloudmqtt.com";
    const PORT: u16 = 11123;
    const USERNAME: &str = "some user";
    const PASSWORD: &str = "some password";

    // cargo test -- --show-output test_plain_client
    #[test]
    #[ignore]
    fn test_plain_client() -> Result<()> {
        let mut rt = tokio::runtime::Runtime::new()?;

        let result = rt.block_on(async {
            let mut c = plain_client(HOST, PORT, USERNAME, PASSWORD)?;
            c.connect().await?;
            println!("connected");
            c.disconnect().await?;
            println!("disconnected");
            Ok(())
        });
        result
    }

    // // cargo test -- --show-output pub_and_sub_plain
    #[test]
    #[ignore]
    fn pub_and_sub_plain() -> Result<()> {
        let mut rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut c = plain_client(HOST, PORT, USERNAME, PASSWORD)?;
            c.connect().await?;

            // Subscribe
            let subopts = Subscribe::new(vec![SubscribeTopic {
                qos: QoS::AtMostOnce,
                topic_path: "test/pub_and_sub".to_owned(),
            }]);
            let subres = c.subscribe(subopts).await?;
            subres.any_failures()?;

            // Publish
            let mut p = Publish::new("test/pub_and_sub".to_owned(), "x".as_bytes().to_vec());
            p.set_qos(QoS::AtMostOnce);
            c.publish(&p).await?;

            // Read
            let r = c.read_subscriptions().await?;
            assert_eq!(r.topic(), "test/pub_and_sub");
            assert_eq!(r.payload(), b"x");
            c.disconnect().await?;
            println!("pub_and_sub_plain OK!");
            Ok(())
        })
    }

    // cargo test -- --show-output sub_only_plain
    #[test]
    #[ignore]
    fn sub_only_plain() -> Result<()> {
        let mut rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut c = plain_client(HOST, PORT, USERNAME, PASSWORD)?;
            c.connect().await?;

            // Subscribe
            let subopts = Subscribe::new(vec![SubscribeTopic {
                qos: QoS::AtMostOnce,
                topic_path: "test/pub_and_sub".to_owned(),
            }]);
            let subres = c.subscribe(subopts).await?;
            subres.any_failures()?;

            // Read
            let r = c.read_subscriptions().await?;
            assert_eq!(r.topic(), "test/pub_and_sub");
            println!("payload is {:#?}", String::from_utf8(r.payload().to_vec()));

            c.disconnect().await?;
            println!("sub_only_plain OK!");
            Ok(())
        })
    }
}
