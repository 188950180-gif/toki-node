//! 账户存储

use crate::{Database, StorageError, CF_ACCOUNTS};
use bincode;
use std::sync::Arc;
use toki_core::{Account, Address};

/// 账户存储
pub struct AccountStore {
    db: Arc<Database>,
}

impl AccountStore {
    /// 创建新账户存储
    pub fn new(db: Arc<Database>) -> Self {
        AccountStore { db }
    }

    /// 保存账户
    pub fn save_account(&self, account: &Account) -> Result<(), StorageError> {
        let key = account.address.as_bytes().to_vec();
        let data = bincode::serialize(account)?;
        self.db.put(CF_ACCOUNTS, &key, &data)?;
        Ok(())
    }

    /// 获取账户
    pub fn get_account(&self, address: &Address) -> Result<Option<Account>, StorageError> {
        let key = address.as_bytes().to_vec();
        let data = self.db.get(CF_ACCOUNTS, &key)?;

        match data {
            Some(bytes) => {
                let account: Account = bincode::deserialize(&bytes)?;
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }

    /// 检查账户是否存在
    pub fn account_exists(&self, address: &Address) -> Result<bool, StorageError> {
        let key = address.as_bytes().to_vec();
        self.db.exists(CF_ACCOUNTS, &key)
    }

    /// 删除账户
    pub fn delete_account(&self, address: &Address) -> Result<(), StorageError> {
        let key = address.as_bytes().to_vec();
        self.db.delete(CF_ACCOUNTS, &key)?;
        Ok(())
    }

    /// 更新账户余额
    pub fn update_balance(
        &self,
        address: &Address,
        new_balance: u64,
    ) -> Result<(), StorageError> {
        let mut account = self
            .get_account(address)?
            .ok_or_else(|| StorageError::NotFound(address.to_string()))?;

        account.balance = new_balance;
        self.save_account(&account)
    }

    /// 批量保存账户
    pub fn save_accounts_batch(&self, accounts: &[Account]) -> Result<(), StorageError> {
        let writes: Vec<crate::WriteOp> = accounts
            .iter()
            .map(|account| {
                let key = account.address.as_bytes().to_vec();
                let data = bincode::serialize(account).unwrap();
                crate::WriteOp::put(CF_ACCOUNTS, key, data)
            })
            .collect();

        self.db.write_batch(writes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use toki_core::{AccountType, TOKI_BASE_UNIT};

    #[test]
    fn test_account_store() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        let store = AccountStore::new(Arc::new(db));

        // 创建测试账户
        let addr = Address::new([1u8; 32]);
        let account = Account::new(addr, AccountType::Personal);

        // 保存
        store.save_account(&account).unwrap();

        // 读取
        let loaded = store.get_account(&addr).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().address, addr);

        // 检查存在
        assert!(store.account_exists(&addr).unwrap());

        // 更新余额
        store.update_balance(&addr, 100 * TOKI_BASE_UNIT).unwrap();
        let loaded = store.get_account(&addr).unwrap().unwrap();
        assert_eq!(loaded.balance, 100 * TOKI_BASE_UNIT);
    }
}
