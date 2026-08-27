use crate::{ChunkStore, MutChunk, RefChunk, World};
use temper_core::dimension::Dimension;
use temper_core::pos::ChunkPos;
use temper_world_format::errors::WorldError;
use temper_world_format::Chunk;
use tracing::trace;
use world_db::chunks::{
    chunk_exists_internal, delete_chunk_internal, load_chunk_internal, save_chunk_internal,
    save_serialized_chunk_internal, sync_internal,
};

struct ChunkSaveSnapshot {
    pos: ChunkPos,
    dimension: Dimension,
    chunk: Chunk,
}

impl ChunkStore {
    pub fn generation_stage(
        &self,
        pos: ChunkPos,
        dimension: Dimension,
    ) -> Result<Option<u8>, WorldError> {
        match self.get_chunk(pos, dimension) {
            Ok(chunk) => Ok(Some(chunk.stage)),
            Err(WorldError::ChunkNotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn ensure_chunk(&self, pos: ChunkPos, dimension: Dimension) -> Result<(), WorldError> {
        // check cache first to avoid nuking io
        if self.cache.contains_key(&(pos, dimension)) {
            return Ok(());
        }

        if self.chunk_exists(pos, dimension)? {
            return Ok(());
        }

        let chunk = Chunk::new_empty();
        chunk.mark_dirty();
        self.cache.insert((pos, dimension), chunk);

        Ok(())
    }

    /// Save a chunk to the storage backend
    ///
    /// This function will save a chunk to the storage backend and update the cache with the new
    /// chunk data. If the chunk already exists in the cache, it will be updated with the new data.
    pub fn insert_chunk(
        &self,
        pos: ChunkPos,
        dimension: Dimension,
        chunk: Chunk,
    ) -> Result<(), WorldError> {
        chunk.clear_dirty();
        save_chunk_internal(&self.storage_backend, pos, dimension, &chunk)?;
        // self.cache.insert((pos, dimension.to_string()), chunk);
        Ok(())
    }

    /// Load a chunk from the storage backend. If the chunk is in the cache, it will be returned
    /// from the cache instead of the storage backend. If the chunk is not in the cache, it will be
    /// loaded from the storage backend and inserted into the cache.
    pub fn get_chunk(
        &'_ self,
        pos: ChunkPos,
        dimension: Dimension,
    ) -> Result<RefChunk<'_>, WorldError> {
        if let Some(chunk) = self.cache.get(&(pos, dimension)) {
            return Ok(chunk);
        }
        let chunk = load_chunk_internal(&self.storage_backend, pos, dimension, self.verify);
        match chunk {
            Ok(c) => {
                self.cache.insert((pos, dimension), c);
                Ok(self
                    .cache
                    .get(&(pos, dimension))
                    .expect("Chunk was just inserted into the cache"))
            }
            Err(e) => Err(e),
        }
    }

    /// Load a mutable chunk from the storage backend. If the chunk is in the cache, it will be returned
    /// from the cache instead of the storage backend. If the chunk is not in the cache, it will be
    /// loaded from the storage backend and inserted into the cache.
    pub fn get_chunk_mut(
        &'_ self,
        pos: ChunkPos,
        dimension: Dimension,
    ) -> Result<MutChunk<'_>, WorldError> {
        if let Some(chunk) = self.cache.get_mut(&(pos, dimension)) {
            return Ok(chunk);
        }
        let chunk = load_chunk_internal(&self.storage_backend, pos, dimension, self.verify);
        match chunk {
            Ok(c) => {
                self.cache.insert((pos, dimension), c);
                Ok(self
                    .cache
                    .get_mut(&(pos, dimension))
                    .expect("Chunk was just inserted into the cache"))
            }
            Err(e) => Err(e),
        }
    }

    /// Check if a chunk exists in the storage backend.
    ///
    /// It will first check if the chunk is in the cache and if it is, it will return true. If the
    /// chunk is not in the cache, it will check the storage backend for the chunk, returning true
    /// if it exists and false if it does not.
    pub fn chunk_exists(&self, pos: ChunkPos, dimension: Dimension) -> Result<bool, WorldError> {
        if self.cache.contains_key(&(pos, dimension)) {
            return Ok(true);
        }
        chunk_exists_internal(&self.storage_backend, pos, dimension)
    }

    /// Delete a chunk from the storage backend.
    ///
    /// This function will remove the chunk from the cache and delete it from the storage backend.
    pub fn delete_chunk(&self, pos: ChunkPos, dimension: Dimension) -> Result<(), WorldError> {
        self.cache.remove(&(pos, dimension));
        delete_chunk_internal(&self.storage_backend, pos, dimension)
    }

    /// Sync the storage backend.
    ///
    /// This function will save all chunks in the cache to the storage backend and then sync the
    /// storage backend. This should be run after inserting or updating a large number of chunks
    /// to ensure that the data is properly saved to disk.
    pub fn sync(&self) -> Result<(), WorldError> {
        let mut snapshots = Vec::new();

        for pair in self.cache.iter() {
            let k = pair.key();
            let v = pair.value();
            if !v.is_dirty() {
                continue;
            }

            trace!("Chunk at {:?} is dirty, saving.", k.0);
            trace!("Syncing chunk: {:?}", k.0);
            if !v.entities.is_empty() {
                trace!(
                    "Chunk at {:?} has {} entities, saving.",
                    k.0,
                    v.entities.len()
                );
            } else {
                trace!("Chunk at {:?} has no entities, saving.", k.0);
            }

            v.clear_dirty();
            snapshots.push(ChunkSaveSnapshot {
                pos: k.0,
                dimension: k.1,
                chunk: v.clone_without_transient_noise(),
            });
        }

        for snapshot in snapshots {
            let data = bitcode::serialize(&snapshot.chunk).expect("Unable to serialize chunk");
            if let Err(err) = save_serialized_chunk_internal(
                &self.storage_backend,
                snapshot.pos,
                snapshot.dimension,
                &data,
            ) {
                if let Ok(chunk) = self.get_chunk(snapshot.pos, snapshot.dimension) {
                    chunk.mark_dirty();
                }

                return Err(err);
            }
        }

        sync_internal(&self.storage_backend)
    }
}

impl World {
    pub fn insert_chunk(
        &self,
        pos: ChunkPos,
        dimension: Dimension,
        chunk: Chunk,
    ) -> Result<(), WorldError> {
        self.chunks.insert_chunk(pos, dimension, chunk)
    }

    pub fn get_chunk(
        &'_ self,
        pos: ChunkPos,
        dimension: Dimension,
    ) -> Result<RefChunk<'_>, WorldError> {
        self.chunks.get_chunk(pos, dimension)
    }

    pub fn get_chunk_mut(
        &'_ self,
        pos: ChunkPos,
        dimension: Dimension,
    ) -> Result<MutChunk<'_>, WorldError> {
        self.chunks.get_chunk_mut(pos, dimension)
    }

    pub fn chunk_exists(&self, pos: ChunkPos, dimension: Dimension) -> Result<bool, WorldError> {
        self.chunks.chunk_exists(pos, dimension)
    }

    pub fn delete_chunk(&self, pos: ChunkPos, dimension: Dimension) -> Result<(), WorldError> {
        self.chunks.delete_chunk(pos, dimension)
    }

    pub fn sync(&self) -> Result<(), WorldError> {
        self.chunks.sync()
    }
}
