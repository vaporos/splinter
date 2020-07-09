// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub(in crate::biome) mod models;
mod operations;
pub(in crate::biome) mod schema;

use super::{User, UserStore, UserStoreError};
use operations::add_user::UserStoreAddUserOperation as _;
use operations::delete_user::UserStoreDeleteUserOperation as _;
use operations::fetch_user::UserStoreFetchUserOperation as _;
use operations::list_users::UserStoreListUsersOperation as _;
use operations::update_user::UserStoreUpdateUserOperation as _;
use operations::UserStoreOperations;

use diesel::sqlite::SqliteConnection;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

/// Manages creating, updating and fetching User from the databae
#[derive(Clone)]
pub struct DieselUserStore<C>
where
    C: diesel::Connection,
    C: 'static,
{
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C> DieselUserStore<C>
where
    C: diesel::Connection,
{
    /// Creates a new DieselUserStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    // Allow dead code if diesel feature is not enabled
    #[allow(dead_code)]
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> DieselUserStore<C> {
        DieselUserStore { connection_pool }
    }
}

impl UserStore for DieselUserStore<PgConnection>
{
    fn add_user(&self, user: User) -> Result<(), UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).add_user(user.into())
    }

    fn update_user(&self, updated_user: User) -> Result<(), UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).update_user(updated_user)
    }

    fn remove_user(&self, id: &str) -> Result<(), UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).delete_user(id)
    }

    fn fetch_user(&self, id: &str) -> Result<User, UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).fetch_user(id)
    }

    fn list_users(&self) -> Result<Vec<User>, UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).list_users()
    }
}

impl UserStore for DieselUserStore<SqliteConnection>
{
    fn add_user(&self, user: User) -> Result<(), UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).add_user(user.into())
    }

    fn update_user(&self, updated_user: User) -> Result<(), UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).update_user(updated_user)
    }

    fn remove_user(&self, id: &str) -> Result<(), UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).delete_user(id)
    }

    fn fetch_user(&self, id: &str) -> Result<User, UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).fetch_user(id)
    }

    fn list_users(&self) -> Result<Vec<User>, UserStoreError> {
        UserStoreOperations::new(&*self.connection_pool.get()?).list_users()
    }
}



#[cfg(test)]
pub mod tests {
    use super::*;

    use diesel::sqlite::SqliteConnection;
    use diesel::r2d2::{ConnectionManager, Pool};

    #[test]
    fn test_add_user() {
        let connection_manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder().build(connection_manager).unwrap();

        let store = DieselUserStore::new(pool);
        store.add_user(User::new("user1")).unwrap();
        store.add_user(User::new("user2")).unwrap();
        store.add_user(User::new("user3")).unwrap();

        assert_eq!(store.fetch_user("user2").unwrap().id(), "user2");
        assert_eq!(store.fetch_user("user1").unwrap().id(), "user2");
        assert_eq!(store.fetch_user("user3").unwrap().id(), "user2");

        assert_eq!(store.list_users().unwrap().len(), 3);
    }
}
