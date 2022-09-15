use cosmwasm_std::{attr, Uint128};

use terrapoker::mock_querier::custom_deps;
use terrapoker::test_constants::governance::{GOVERNANCE, governance_env, GOVERNANCE_TOKEN};

use crate::staking::queries::get_staker_state;
use crate::staking::tests::stake_token_hook::STAKER1;
use crate::tests::init_default;

#[test]
fn share_calculation() {
    let mut deps = custom_deps();

    init_default(deps.as_mut());

    super::stake_token_hook::will_success(&mut deps, STAKER1, Uint128::new(100));

    deps.querier.plus_token_balances(&[(
        GOVERNANCE_TOKEN,
        &[(GOVERNANCE, &Uint128::new(100))],
    )]);

    let (_, _, response) = super::stake_token_hook::will_success(
        &mut deps,
        STAKER1,
        Uint128::new(100),
    );

    assert_eq!(response.attributes, vec![
        attr("action", "stake_token_hook"),
        attr("sender", STAKER1),
        attr("share", "50"),
        attr("amount", "100"),
    ]);

    let (_, _, response) = super::unstake_token_hook::will_success(
        &mut deps,
        STAKER1,
        Some(Uint128::new(100)),
    );

    assert_eq!(response.attributes, vec![
        attr("action", "unstake_token_hook"),
        attr("unstake_amount", "100"),
        attr("unstake_share", "50")
    ]);

    let staker_state = get_staker_state(deps.as_ref(), governance_env(), STAKER1.to_string()).unwrap();
    assert_eq!(staker_state.share, Uint128::new(100));
    assert_eq!(staker_state.balance, Uint128::new(200));
}
