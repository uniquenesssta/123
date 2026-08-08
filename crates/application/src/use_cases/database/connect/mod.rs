use crate::ports::{database::DatabaseLifecyclePort, PortResult};

pub(crate) async fn execute(port: &(impl DatabaseLifecyclePort + ?Sized)) -> PortResult<()> {
    super::migrate::execute(port).await?;
    port.recover_interrupted_work().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeDatabase {
        events: Mutex<Vec<&'static str>>,
    }

    #[async_trait]
    impl DatabaseLifecyclePort for FakeDatabase {
        async fn migrate(&self) -> PortResult<()> {
            self.events.lock().expect("events lock").push("migrate");
            Ok(())
        }

        async fn recover_interrupted_work(&self) -> PortResult<()> {
            self.events.lock().expect("events lock").push("recover");
            Ok(())
        }

        async fn reset_to_pristine(&self) -> PortResult<()> {
            self.events.lock().expect("events lock").push("reset");
            Ok(())
        }

        async fn close(&self) -> PortResult<()> {
            self.events.lock().expect("events lock").push("close");
            Ok(())
        }
    }

    #[tokio::test]
    async fn connection_preparation_migrates_before_recovery() {
        let database = FakeDatabase::default();
        execute(&database).await.expect("connect use case");
        assert_eq!(
            database.events.lock().expect("events lock").as_slice(),
            ["migrate", "recover"]
        );
    }
}
