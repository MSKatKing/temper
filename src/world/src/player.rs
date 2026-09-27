use crate::World;
use serde::de::DeserializeOwned;
use serde::Serialize;
use temper_world_format::errors::WorldError;
use tracing::trace;

impl World {
    /// Loads player data from the storage backend and decodes it.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A type that can be deserialized with serde.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The unique identifier of the player whose data is to be loaded.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(T))` - The decoded player data if it exists and can be successfully decoded.
    /// * `Ok(None)` - If no data is found for the given player.
    /// * `Err(WorldError)` - If an error occurs during the operation or decoding fails.
    pub fn load_player_data<T: DeserializeOwned>(
        &self,
        uuid: uuid::Uuid,
    ) -> Result<Option<T>, WorldError> {
        if !self
            .chunks
            .storage_backend
            .table_exists("player_data".to_string())?
        {
            trace!(
                "Player data table does not exist. Returning None for player {}",
                uuid
            );
            return Ok(None);
        }
        let data = self
            .chunks
            .storage_backend
            .get("player_data".to_string(), uuid.as_u128())
            .map_err(WorldError::DatabaseError);
        data.and_then(|opt_bytes| {
            opt_bytes
                .map(|bytes| {
                    bitcode::deserialize(&bytes)
                        .map_err(|err| WorldError::BitcodeDeserializeError(err.to_string()))
                })
                .transpose()
        })
    }

    /// Saves player data to the storage backend after encoding it.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A type that can be serialized with serde.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The unique identifier of the player whose data is to be saved.
    /// * `data` - A reference to the data to be encoded and saved.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the data was successfully saved.
    /// * `Ok(false)` - If the data could not be saved.
    /// * `Err(WorldError)` - If an error occurs during the operation.
    pub fn save_player_data<T: Serialize>(
        &self,
        uuid: uuid::Uuid,
        data: &T,
    ) -> Result<bool, WorldError> {
        if !self
            .chunks
            .storage_backend
            .table_exists("player_data".to_string())?
        {
            self.chunks
                .storage_backend
                .create_table("player_data".to_string())
                .map_err(WorldError::DatabaseError)?;
        }
        self.chunks
            .storage_backend
            .upsert(
                "player_data".to_string(),
                uuid.as_u128(),
                bitcode::serialize(data)
                    .map_err(|err| WorldError::BitcodeSerializeError(err.to_string()))?,
            )
            .map_err(WorldError::DatabaseError)
    }
}
