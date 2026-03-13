use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};
use crate::entity::prelude::*;
use crate::entity::user;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Contributor,
    Admin,
    Root,
}

impl Role {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "root" => Role::Root,
            "admin" => Role::Admin,
            "contributor" => Role::Contributor,
            _ => Role::User,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Role::Root => "root",
            Role::Admin => "admin",
            Role::Contributor => "contributor",
            Role::User => "user",
        }
    }

    pub fn is_at_least(&self, other: Role) -> bool {
        match (self, other) {
            (Role::Root, _) => true,
            (Role::Admin, Role::Root) => false,
            (Role::Admin, _) => true,
            (Role::Contributor, Role::Root) => false,
            (Role::Contributor, Role::Admin) => false,
            (Role::Contributor, _) => true,
            (Role::User, Role::User) => true,
            _ => false,
        }
    }

    pub fn is_root(&self) -> bool {
        matches!(self, Role::Root)
    }

    pub fn is_admin_or_higher(&self) -> bool {
        self.is_at_least(Role::Admin)
    }

    pub fn is_contributor_or_higher(&self) -> bool {
        self.is_at_least(Role::Contributor)
    }
}

pub struct UserClaims {
    pub user_id: i32,
    pub role: Role,
}

impl UserClaims {
    pub fn new(user_id: i32, role_str: &str) -> Self {
        Self {
            user_id,
            role: Role::from_str(role_str),
        }
    }

    pub fn is_root(&self) -> bool {
        self.role.is_root()
    }

    pub fn is_admin_or_higher(&self) -> bool {
        self.role.is_admin_or_higher()
    }

    pub fn is_contributor_or_higher(&self) -> bool {
        self.role.is_contributor_or_higher()
    }
}

pub async fn is_root_user(state: &AppState, user_id: i32) -> bool {
    if let Ok(Some(user)) = User::find_by_id(user_id).one(&state.db).await {
        user.role == "root"
    } else {
        false
    }
}

pub async fn has_only_one_root(state: &AppState) -> bool {
    if let Ok(count) = User::find()
        .filter(user::Column::Role.eq("root"))
        .count(&state.db)
        .await
    {
        count <= 1
    } else {
        false
    }
}

pub async fn can_manage_user(
    state: &AppState,
    current_user_id: i32,
    current_user_role: &str,
    target_user_id: i32,
) -> bool {
    let current_role = Role::from_str(current_user_role);
    
    if current_role == Role::Root {
        return true;
    }
    
    if current_role != Role::Admin {
        return false;
    }
    
    if current_user_id == target_user_id {
        return false;
    }
    
    if let Ok(Some(target_user)) = User::find_by_id(target_user_id).one(&state.db).await {
        let target_role = Role::from_str(&target_user.role);
        !target_role.is_at_least(Role::Admin)
    } else {
        false
    }
}

pub async fn can_change_user_role(
    state: &AppState,
    current_user_id: i32,
    current_user_role: &str,
    target_user_id: i32,
    new_role: &str,
) -> bool {
    let current_role = Role::from_str(current_user_role);
    let new_role_enum = Role::from_str(new_role);
    
    if current_role == Role::Root {
        return true;
    }
    
    if current_role != Role::Admin {
        return false;
    }
    
    if current_user_id == target_user_id {
        return false;
    }
    
    if let Ok(Some(target_user)) = User::find_by_id(target_user_id).one(&state.db).await {
        let target_role = Role::from_str(&target_user.role);
        if target_role.is_at_least(Role::Admin) {
            return false;
        }
    } else {
        return false;
    }
    
    !new_role_enum.is_at_least(Role::Admin)
}

pub async fn can_manage_book(
    state: &AppState,
    current_user_id: i32,
    current_user_role: &str,
    book_id: i32,
) -> bool {
    let current_role = Role::from_str(current_user_role);
    
    if current_role.is_admin_or_higher() {
        return true;
    }
    
    if current_role != Role::Contributor {
        return false;
    }
    
    if let Ok(Some(book)) = Book::find_by_id(book_id).one(&state.db).await {
        book.created_by == Some(current_user_id)
    } else {
        false
    }
}

pub async fn can_manage_group(
    state: &AppState,
    current_user_id: i32,
    current_user_role: &str,
    group_id: i32,
) -> bool {
    let current_role = Role::from_str(current_user_role);
    
    if current_role.is_admin_or_higher() {
        return true;
    }
    
    if current_role != Role::Contributor {
        return false;
    }
    
    if let Ok(Some(group)) = Group::find_by_id(group_id).one(&state.db).await {
        group.created_by == Some(current_user_id)
    } else {
        false
    }
}
