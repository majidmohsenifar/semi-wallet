use clap::Args;

use crate::service::blockchain::service::Service as BlockchainService;
use crate::service::user_coin::service::Service as UserCoinService;
use crate::service::user_plan::service::Service as UserPlanService;

//pub const UPDATE_USERS_COINS_AMOUNT_COMMAND: &str = "update-users-coins-amount";

#[derive(Debug, Args)]
#[command(flatten_help = true)]
pub struct UpdateUserCoinsAmountArgs {
    #[arg(short, long)]
    user_id: Option<i64>,
    #[arg(short, long)]
    symbol: Option<String>,
}

pub struct UpdateUserCoinsCommand {
    user_coin_service: UserCoinService,
    user_plan_service: UserPlanService,
    blockchain_service: BlockchainService,
}

impl UpdateUserCoinsCommand {
    pub fn new(
        user_coin_service: UserCoinService,
        user_plan_service: UserPlanService,
        blockchain_service: BlockchainService,
    ) -> Self {
        UpdateUserCoinsCommand {
            user_coin_service,
            user_plan_service,
            blockchain_service,
        }
    }

    pub async fn run(&self, args: UpdateUserCoinsAmountArgs) {
        //TODO: handle args later
        let mut last_id = 0;
        let page_size = 100;

        loop {
            let user_plans = self
                .user_plan_service
                .get_non_expired_users_plans(last_id, page_size)
                .await;
            let user_plans = match user_plans {
                Err(e) => {
                    tracing::error!("cannot get_non_expired_users_plans due to err: {}", e);
                    break;
                }
                Ok(data) => data,
            };
            if user_plans.is_empty() {
                break;
            }

            let user_ids: Vec<i64> = user_plans
                .iter()
                .map(|user_plan| user_plan.user_id)
                .collect();

            let user_coins = self
                .user_coin_service
                .get_user_coins_by_user_ids(user_ids)
                .await;

            let user_coins = match user_coins {
                Err(e) => {
                    tracing::error!("cannot get_non_expired_users_plans due to err: {}", e);
                    break;
                }
                Ok(data) => data,
            };

            for uc in user_coins {
                let balance = self
                    .blockchain_service
                    .get_balance(&uc.network, &uc.symbol, &uc.address)
                    .await;
                let balance = match balance {
                    Err(e) => {
                        tracing::error!(
                            "cannot get_balance for coin {}, network {}, user_id {}, due to err: {}",
                            uc.symbol,
                            uc.network,
                            uc.user_id,
                            e
                        );
                        continue;
                    }
                    Ok(b) => b,
                };
                let update_res = self
                    .user_coin_service
                    .update_user_coin_amount(uc.id, balance)
                    .await;
                if let Err(e) = update_res {
                    tracing::error!(
                        "cannot update user amount for coin {}, network {}, user_id {}, due to err: {}",
                        uc.symbol,
                        uc.network,
                        uc.user_id,
                        e
                    );
                    continue;
                }
            }

            if (user_plans.len() as i64) < page_size {
                break;
            }

            //we are sure the last would exist, as if it does not, we would break before this line,
            //so unwrap is ok to use
            last_id = user_plans.last().unwrap().id;
        }
    }
}
