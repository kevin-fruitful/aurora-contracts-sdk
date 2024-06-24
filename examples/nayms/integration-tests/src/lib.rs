use aurora_sdk_integration_tests::{
    aurora_engine, aurora_engine_types::U256, tokio, utils, workspaces,
};
use nayms_on_near_types::Entity;

const ENTITY_ID: &str = "0000000000000000000000000000000000000000000000000000000000000001";

#[tokio::test]
async fn test_nayms_proxy_contract() {
    let worker = workspaces::sandbox().await.unwrap();
    let engine = aurora_engine::deploy_latest(&worker).await.unwrap();

    // Build and deploy the NaymsProxy contract
    tokio::fs::create_dir_all("../target/near/nayms_on_near")
        .await
        .unwrap();
    let contract_bytes = utils::cargo::build_contract("../near-contract").await.unwrap();
    let contract = contract_interface::NaymsProxy {
        contract: worker.dev_deploy(&contract_bytes).await.unwrap(),
    };

    // Deploy mock Nayms contract on Aurora
    let nayms_contract = deploy_mock_nayms_contract(&engine).await;

    // Initialize our NaymsProxy contract
    contract
        .create(engine.inner.id(), &nayms_contract.address().encode())
        .await
        .unwrap();

    // Call get_entity through our NaymsProxy contract
    let entity = contract.get_entity(ENTITY_ID).await.unwrap();

    // Assert the returned entity matches what we expect
    assert_eq!(entity.asset_id.as_ref(), hex::decode(ENTITY_ID).unwrap());
    assert_eq!(entity.collateral_ratio, U256::from(1000)); // Assuming 100% collateral ratio
    assert_eq!(entity.max_capacity, U256::from(1_000_000));
    assert_eq!(entity.utilized_capacity, U256::from(500_000));
    assert_eq!(entity.simple_policy_enabled, true);
}

// This function would deploy a mock Nayms contract to Aurora for testing purposes
async fn deploy_mock_nayms_contract(engine: &aurora_engine::AuroraEngine) -> MockNaymsContract {
    // Implementation depends on how you want to mock the Nayms contract
    // For this example, we'll just return a dummy contract
    MockNaymsContract {
        address: aurora_engine_types::types::Address::decode(&[0; 20]).unwrap(),
    }
}

struct MockNaymsContract {
    address: aurora_engine_types::types::Address,
}

impl MockNaymsContract {
    fn address(&self) -> aurora_engine_types::types::Address {
        self.address
    }
}

// This module contains convenience functions for interacting with the NaymsProxy contract
mod contract_interface {
    use aurora_sdk_integration_tests::workspaces::{self, Contract};
    use nayms_on_near_types::Entity;

    pub struct NaymsProxy {
        pub contract: Contract,
    }

    impl NaymsProxy {
        pub async fn create(
            &self,
            aurora: &workspaces::AccountId,
            nayms_address: &str,
        ) -> Result<(), workspaces::error::Error> {
            let result = self
                .contract
                .call("new")
                .args_json(NewArgs {
                    aurora,
                    nayms_address,
                })
                .max_gas()
                .transact()
                .await?;
            result.into_result()?;
            Ok(())
        }

        pub async fn get_entity(&self, entity_id: &str) -> Result<Entity, workspaces::error::Error> {
            let result = self
                .contract
                .call("get_entity")
                .args_json(GetEntityArgs { entity_id })
                .max_gas()
                .transact()
                .await?;
            
            result.json().map_err(Into::into)
        }
    }

    #[derive(serde::Serialize)]
    struct NewArgs<'a> {
        aurora: &'a workspaces::AccountId,
        nayms_address: &'a str,
    }

    #[derive(serde::Serialize)]
    struct GetEntityArgs<'a> {
        entity_id: &'a str,
    }
}