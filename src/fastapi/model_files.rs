use anyhow::Result;

use crate::{file_manager::save_file_with_content, project_info::ProjectInfo};

fn create_message_model_file() -> String {
    r#"from __future__ import annotations

from camel_converter.pydantic_base import CamelBase


class Message(CamelBase):
    """Used for generic messages."""

    message: str
"#
    .to_string()
}

pub fn save_message_model_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("models/message.py");
    let file_content = create_message_model_file();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_token_models_file() -> String {
    r#"from pydantic import BaseModel


class Token(BaseModel):
    """Don't use CamelBase here because FastAPI requires snake case vairables for the token."""

    access_token: str
    token_type: str = "bearer"


class TokenPayload(BaseModel):
    """Contents of the JWT token."""

    sub: str | None = None
    is_superuser: bool = False
"#
    .to_string()
}

pub fn save_token_models_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("models/token.py");
    let file_content = create_token_models_file();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}

fn create_user_models_file() -> String {
    r#"from __future__ import annotations

import re
from datetime import datetime

from camel_converter.pydantic_base import CamelBase
from pydantic import EmailStr, Field, field_validator


class _UserBase(CamelBase):
    email: EmailStr = Field(max_length=255)
    is_active: bool = True
    is_superuser: bool = False
    full_name: str = Field(max_length=255)


class UserCreate(_UserBase):
    password: str = Field(min_length=8, max_length=255)

    @field_validator("password")
    @classmethod
    def validate_password_requirements(cls, v: str) -> str:
        return _validate_password(v)


class UserUpdate(CamelBase):
    email: EmailStr | None = Field(default=None, max_length=255)
    is_active: bool | None = None
    is_superuser: bool | None = None
    password: str | None = Field(default=None, min_length=8, max_length=255)
    full_name: str | None = Field(default=None, max_length=255)

    @field_validator("password")
    @classmethod
    def validate_password_requirements(cls, v: str) -> str:
        return _validate_password(v)


class UserUpdateMe(CamelBase):
    email: EmailStr | None = Field(default=None, max_length=255)
    full_name: str | None = Field(default=None, max_length=255)


class UpdatePassword(CamelBase):
    current_password: str = Field(min_length=8, max_length=255)
    new_password: str = Field(min_length=8, max_length=255)


class User(_UserBase):
    id: str
    hashed_password: str


class UserStudy(CamelBase):
    user_id: str
    study_id: str


class UserStudyPublic(CamelBase):
    id: str = Field(max_length=255)
    is_active: bool
    created: datetime


class UserPublic(_UserBase):
    id: str


class UsersPublic(CamelBase):
    data: list[UserPublic]
    count: int
    total_users: int


class UserInDb(_UserBase):
    id: str
    hashed_password: str
    last_login: datetime


def _validate_password(password: str) -> str:
    """Makes sure the password meets the minimum requirements.

    Passwords must to contain at least 1 uppercase letter, 1 lowercase letter, a number, and a
    special character. They must be a minimum of 8 characters.
    """
    if (
        not (
            re.search(r"[A-Z]", password)
            and re.search(r"[a-z]", password)
            and re.search(r"\d", password)
            and re.search(r"[!@#$%^&*()_+\-=\[\]{};':\"\\|,.<>\/?]", password)
        )
        or len(password) < 8
    ):
        raise ValueError(
            "Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character. They must be a minimum of 8 characters."
        )
    return password
"#.to_string()
}

pub fn save_user_models_file(project_info: &ProjectInfo) -> Result<()> {
    let base = &project_info.source_dir_path();
    let file_path = base.join("models/users.py");
    let file_content = create_user_models_file();

    save_file_with_content(&file_path, &file_content)?;

    Ok(())
}
