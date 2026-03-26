use crate::{AppResult, use_cases::UseCase};

pub struct CreateContextObjectCommand;

pub struct CreateContextObjectResponse;

pub struct CreateContextObjectUseCase {}

#[async_trait::async_trait]
impl UseCase<CreateContextObjectCommand, CreateContextObjectResponse>
    for CreateContextObjectUseCase
{
    async fn execute(
        &self,
        command: CreateContextObjectCommand,
    ) -> AppResult<CreateContextObjectResponse> {
        todo!()
    }
}
