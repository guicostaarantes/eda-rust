use crate::domain::astronaut_commander::AstronautCommander;
use crate::domain::astronaut_commander::AstronautCommanderError;
use crate::domain::astronaut_model::CreateAstronautInput;
use crate::domain::astronaut_model::UpdateAstronautInput;
use crate::domain::token_commander::TokenCommander;
use crate::domain::token_commander::TokenCommanderError;
use crate::domain::token_model::AstronautCredentialsInput;
use crate::domain::token_model::TokenPair;
use crate::providers::token::RawToken;
use async_graphql::Context;
use async_graphql::Object;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_astronaut(
        &self,
        ctx: &Context<'_>,
        input: CreateAstronautInput,
    ) -> Result<String, AstronautCommanderError> {
        ctx.data_unchecked::<AstronautCommander>()
            .create_astronaut(input)
            .await
    }

    async fn update_astronaut(
        &self,
        ctx: &Context<'_>,
        id: String,
        input: UpdateAstronautInput,
    ) -> Result<String, AstronautCommanderError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                ctx.data_unchecked::<AstronautCommander>()
                    .update_astronaut(raw_token, id, input)
                    .await
            }
            None => Err(AstronautCommanderError::Forbidden),
        }
    }

    async fn exchange_astronaut_credentials_for_token_pair(
        &self,
        ctx: &Context<'_>,
        input: AstronautCredentialsInput,
    ) -> Result<TokenPair, TokenCommanderError> {
        ctx.data_unchecked::<TokenCommander>()
            .exchange_astronaut_credentials_for_token_pair(input)
            .await
    }

    async fn refresh_token_pair(
        &self,
        ctx: &Context<'_>,
    ) -> Result<TokenPair, TokenCommanderError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                ctx.data_unchecked::<TokenCommander>()
                    .refresh_token_pair(&raw_token)
                    .await
            }
            None => Err(TokenCommanderError::Forbidden),
        }
    }

    async fn invalidate_refresh_token(
        &self,
        ctx: &Context<'_>,
    ) -> Result<String, TokenCommanderError> {
        match ctx.data_opt::<RawToken>() {
            Some(raw_token) => {
                ctx.data_unchecked::<TokenCommander>()
                    .invalidate_refresh_token(&raw_token)
                    .await
            }
            None => Err(TokenCommanderError::Forbidden),
        }?;

        Ok("ok".to_string())
    }
}
