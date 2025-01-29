#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{InstantiateMsg, ExecuteMsg};
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    /// Returns a boxed instance of the contract to be used in testing.
    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    /// Mocks the blockchain environment and initializes the user's balance.
    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &MockApi::default().addr_validate(USER).unwrap(),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    /// Instantiates the contract and ensures proper initialization.
    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let user = app.api().addr_validate(USER).unwrap();
        assert_eq!(
            app.wrap().query_balance(user.clone(), NATIVE_DENOM).unwrap().amount,
            Uint128::new(1)
        );

        let msg = InstantiateMsg {
            name: "SEINT".to_string(),
            symbol: "SEINT".to_string(),
            decimals: 6,
            initial_supply: Uint128::new(1_000_000_000_000_000),
            specified_address: "0x45d2B456361f5D2D3e473018E56482059075eceB".to_string(),
        };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    mod token_tests {
        use super::*;

        #[test]
        fn test_transfer() {
            let (mut app, cw_template_contract) = proper_instantiate();

            // Transfer 100 units from USER to another address
            let recipient = "recipient".to_string();
            let msg = ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount: Uint128::new(100),
            };

            let cosmos_msg = cw_template_contract.call(msg).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

            // Check balances
            let recipient_balance: Uint128 = app
                .wrap()
                .query_wasm_smart(
                    cw_template_contract.addr(),
                    &crate::msg::QueryMsg::GetBalance { address: recipient },
                )
                .unwrap();
            assert_eq!(recipient_balance, Uint128::new(100));

            let user_balance: Uint128 = app
                .wrap()
                .query_wasm_smart(
                    cw_template_contract.addr(),
                    &crate::msg::QueryMsg::GetBalance {
                        address: USER.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(
                user_balance,
                Uint128::new(800_000_000_000_000 - 100) // 80% of total supply minus transferred amount
            );
        }
    }
}
