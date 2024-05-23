use crate::soul::WonderingSoul;
use crate::world::World;

use crate::api::error;
use crate::api::error::Result;

// Verify soul authority
pub async fn verify_soul(world: &World, uid: &String, pw_hash: Option<String>) -> Result<()> {
    let pw_hash = pw_hash.ok_or(error::ApiError::EmptyPassword)?;
    if !world.verify_soul(uid, &pw_hash).await? {
        Err(error::ApiError::AuthError(uid.clone()).into())
    } else {
        Ok(())
    }
}

// Verify soul authority and return it
pub async fn get_verified_soul<'w>(
    world: &'w World,
    uid: &String,
    pw_hash: Option<String>,
) -> Result<WonderingSoul<'w>> {
    let pw_hash = pw_hash.ok_or(error::ApiError::EmptyPassword)?;
    if !world.verify_soul(uid, &pw_hash).await? {
        Err(error::ApiError::AuthError(uid.clone()).into())
    } else {
        Ok(world.get_wondering_soul(uid).await?.unwrap()) // Soul 注册后不会删除，所以可以直接返回
    }
}
