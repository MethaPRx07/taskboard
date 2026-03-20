# Calendar Task Management API — Documentation

> **Stack:** Rust · Actix-web 4 · SQLx 0.8 · PostgreSQL · JWT (HS256) · Argon2

---

## Table of Contents
1. [Setup & Running](#setup--running)
2. [Database Schema](#database-schema)
3. [Authentication](#authentication)
4. [API Endpoints](#api-endpoints)
   - [Auth](#auth-endpoints)
   - [Calendars](#calendar-endpoints)
   - [Tasks](#task-endpoints)
5. [Error Responses](#error-responses)

---

## Setup & Running

### 1. Prerequisites
- Rust 1.75+
- PostgreSQL 14+
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli) (optional, for manual migrations)

### 2. Environment
```bash
cp .env.example .env
# Edit .env with your DB credentials and JWT secret
```

### 3. Run
```bash
cargo run
# Server starts on http://127.0.0.1:3000
# Migrations run automatically on startup
```

### 4. Manual Migrations (optional)
```bash
cargo install sqlx-cli --features postgres
sqlx migrate run
```

---

## Database Schema

```
┌─────────────────┐        ┌──────────────────────┐
│     users       │        │    refresh_tokens     │
├─────────────────┤        ├──────────────────────┤
│ id (PK)         │──┐     │ id (PK)               │
│ email (UNIQUE)  │  └────▶│ user_id (FK)          │
│ name            │        │ token_hash (UNIQUE)   │
│ password_hash   │        │ expires_at            │
│ role            │        │ created_at            │
│ is_active       │        └──────────────────────┘
│ created_at      │
│ updated_at      │
└────────┬────────┘
         │
         │ owner_id
         ▼
┌─────────────────┐        ┌──────────────────────┐
│   calendars     │        │  calendar_members    │
├─────────────────┤        ├──────────────────────┤
│ id (PK)         │──┐     │ calendar_id (FK, PK) │
│ owner_id (FK)   │  └────▶│ user_id (FK, PK)     │
│ name            │        │ role: owner│editor│  │
│ description     │        │       viewer          │
│ color           │        │ joined_at             │
│ is_public       │        └──────────────────────┘
│ created_at      │
│ updated_at      │
└────────┬────────┘
         │ calendar_id
         ▼
┌─────────────────┐     ┌───────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│     tasks       │     │  task_assignees   │     │  task_labels     │     │  task_comments   │
├─────────────────┤     ├───────────────────┤     ├──────────────────┤     ├──────────────────┤
│ id (PK)         │──┬─▶│ task_id (FK, PK)  │     │ task_id (FK, PK) │     │ id (PK)          │
│ calendar_id (FK)│  │  │ user_id (FK, PK)  │  ┌─▶│ label (PK)       │  ┌─▶│ task_id (FK)     │
│ creator_id (FK) │  │  │ assigned_at       │  │  │ color            │  │  │ user_id (FK)     │
│ title           │  ├──┘                   │  │  └──────────────────┘  │  │ content          │
│ description     │  ├──────────────────────┘  │                        │  │ created_at       │
│ status          │  └──────────────────────────┘                        │  │ updated_at       │
│ priority        │  └──────────────────────────────────────────────────┘  └──────────────────┘
│ due_date        │
│ start_date      │
│ all_day         │
│ created_at      │
│ updated_at      │
└─────────────────┘
```

### Enums

| Field | Allowed Values |
|-------|---------------|
| `users.role` | `user`, `admin` |
| `calendar_members.role` | `owner`, `editor`, `viewer` |
| `tasks.status` | `todo`, `in_progress`, `done`, `cancelled` |
| `tasks.priority` | `low`, `medium`, `high`, `urgent` |

---

## Authentication

The API uses **JWT (HS256)** access tokens + opaque refresh tokens.

- **Access token**: short-lived (default 15 min), sent as `Authorization: Bearer <token>`
- **Refresh token**: long-lived (default 7 days), stored as SHA-256 hash in DB

### Flow

```
Client                          API
  │─── POST /auth/register ───▶│ Create user
  │◀── { access_token,         │
  │      refresh_token, user } ─│

  │─── POST /auth/login ──────▶│ Verify password
  │◀── { access_token,         │
  │      refresh_token, user } ─│

  │─── GET /auth/me ──────────▶│ Authorization: Bearer <access_token>
  │◀── { user }                │

  │─── POST /auth/refresh ────▶│ { refresh_token }
  │◀── { access_token }        │ (Get new access token)

  │─── POST /auth/logout ─────▶│ Authorization: Bearer <token>
  │                             │ { refresh_token }  → revoke token
  │◀── 204 No Content          │
```

---

## API Endpoints

**Base URL:** `http://localhost:3000/api/v1`

### Auth Endpoints

#### `POST /auth/register`
Register a new user. Returns access + refresh tokens.

**Request Body:**
```json
{
  "email": "user@example.com",
  "name": "John Doe",
  "password": "securepassword123"
}
```

**Response `201 Created`:**
```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "a3f9b2c1d4e5...",
  "token_type": "Bearer",
  "expires_in": 900,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe",
    "role": "user",
    "is_active": true,
    "created_at": "2026-03-11T07:30:00Z"
  }
}
```

---

#### `POST /auth/login`
Login with email and password.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "securepassword123"
}
```

**Response `200 OK`:** Same as register response.

---

#### `POST /auth/refresh`
Get a new access token using a refresh token.

**Request Body:**
```json
{ "refresh_token": "a3f9b2c1d4e5..." }
```

**Response `200 OK`:**
```json
{
  "access_token": "eyJhbGc...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

---

#### `POST /auth/logout` 🔒
Revoke a refresh token.

**Headers:** `Authorization: Bearer <access_token>`

**Request Body:**
```json
{ "refresh_token": "a3f9b2c1d4e5..." }
```

**Response `204 No Content`**

---

#### `GET /auth/me` 🔒
Get current user profile.

**Headers:** `Authorization: Bearer <access_token>`

**Response `200 OK`:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "name": "John Doe",
  "role": "user",
  "is_active": true,
  "created_at": "2026-03-11T07:30:00Z"
}
```

---

### Calendar Endpoints

All calendar endpoints require 🔒 `Authorization: Bearer <access_token>`.

#### `POST /calendars`
Create a new calendar. Creator becomes the `owner` member automatically.

**Request Body:**
```json
{
  "name": "Work Calendar",
  "description": "Team schedule and deadlines",
  "color": "#4285F4",
  "is_public": false
}
```

**Response `201 Created`:**
```json
{
  "id": "a1b2c3d4-...",
  "owner_id": "550e8400-...",
  "name": "Work Calendar",
  "description": "Team schedule and deadlines",
  "color": "#4285F4",
  "is_public": false,
  "created_at": "2026-03-11T07:30:00Z",
  "updated_at": "2026-03-11T07:30:00Z"
}
```

---

#### `GET /calendars`
List all calendars where user is owner or member.

**Response `200 OK`:** Array of Calendar objects.

---

#### `GET /calendars/{id}`
Get a specific calendar. Public calendars are accessible without membership.

**Response `200 OK`:** Calendar object.

---

#### `PUT /calendars/{id}`
Update calendar. Requires `owner` or `editor` role.

**Request Body:** (all fields optional)
```json
{
  "name": "Updated Name",
  "description": "New description",
  "color": "#FF5733",
  "is_public": true
}
```

**Response `200 OK`:** Updated Calendar object.

---

#### `DELETE /calendars/{id}`
Delete calendar and all its tasks. Requires `owner` role.

**Response `204 No Content`**

---

#### `POST /calendars/{id}/members`
Add a member to the calendar. Requires `owner` role.

**Request Body:**
```json
{
  "user_id": "550e8400-...",
  "role": "editor"
}
```
> `role` must be `editor` or `viewer` (cannot assign `owner`)

**Response `201 Created`:**
```json
{
  "calendar_id": "a1b2c3d4-...",
  "user_id": "550e8400-...",
  "role": "editor",
  "joined_at": "2026-03-11T07:30:00Z"
}
```

---

#### `DELETE /calendars/{id}/members/{user_id}`
Remove a member. Owner can remove anyone; members can remove themselves.

**Response `204 No Content`**

---

### Task Endpoints

All task endpoints require 🔒 `Authorization: Bearer <access_token>`.

#### `POST /calendars/{calendar_id}/tasks`
Create a task in a calendar.

**Request Body:**
```json
{
  "title": "Design mockups",
  "description": "Create Figma designs for the new landing page",
  "status": "todo",
  "priority": "high",
  "due_date": "2026-03-20T17:00:00Z",
  "start_date": "2026-03-15T09:00:00Z",
  "all_day": false
}
```

**Response `201 Created`:**
```json
{
  "id": "t1a2b3c4-...",
  "calendar_id": "a1b2c3d4-...",
  "creator_id": "550e8400-...",
  "title": "Design mockups",
  "description": "Create Figma designs for the new landing page",
  "status": "todo",
  "priority": "high",
  "due_date": "2026-03-20T17:00:00Z",
  "start_date": "2026-03-15T09:00:00Z",
  "all_day": false,
  "created_at": "2026-03-11T07:30:00Z",
  "updated_at": "2026-03-11T07:30:00Z"
}
```

---

#### `GET /calendars/{calendar_id}/tasks`
List tasks with optional filters.

**Query Parameters:**

| Param | Type | Description |
|-------|------|-------------|
| `status` | string | Filter by status: `todo`, `in_progress`, `done`, `cancelled` |
| `priority` | string | Filter by priority: `low`, `medium`, `high`, `urgent` |
| `date_from` | ISO 8601 | Tasks with `due_date >= date_from` |
| `date_to` | ISO 8601 | Tasks with `due_date <= date_to` |
| `assignee` | UUID | Tasks assigned to a specific user |

**Example:**
```
GET /api/v1/calendars/{id}/tasks?status=todo&priority=high&date_from=2026-03-01T00:00:00Z
```

**Response `200 OK`:** Array of Task objects.

---

#### `GET /tasks/{id}`
Get a specific task.

**Response `200 OK`:** Task object.

---

#### `PUT /tasks/{id}`
Update a task (partial update, all fields optional).

**Request Body:**
```json
{
  "title": "Updated title",
  "priority": "urgent",
  "due_date": "2026-03-18T17:00:00Z"
}
```

**Response `200 OK`:** Updated Task object.

---

#### `DELETE /tasks/{id}`
Delete a task. Only task creator or calendar owner can delete.

**Response `204 No Content`**

---

#### `PATCH /tasks/{id}/status`
Update only the task status.

**Request Body:**
```json
{ "status": "in_progress" }
```

**Response `200 OK`:** Updated Task object.

---

#### `POST /tasks/{id}/assignees`
Assign a user to the task.

**Request Body:**
```json
{ "user_id": "550e8400-..." }
```

**Response `201 Created`:**
```json
{
  "task_id": "t1a2b3c4-...",
  "user_id": "550e8400-...",
  "assigned_at": "2026-03-11T07:30:00Z"
}
```

---

#### `DELETE /tasks/{id}/assignees/{user_id}`
Remove a user assignment from the task.

**Response `204 No Content`**

---

#### `POST /tasks/{id}/labels`
Add a label to the task.

**Request Body:**
```json
{
  "label": "frontend",
  "color": "#3498DB"
}
```

**Response `201 Created`:**
```json
{
  "task_id": "t1a2b3c4-...",
  "label": "frontend",
  "color": "#3498DB"
}
```

---

#### `DELETE /tasks/{id}/labels/{label}`
Remove a label from the task.

**Response `204 No Content`**

---

#### `POST /tasks/{id}/comments`
Add a comment to the task.

**Request Body:**
```json
{ "content": "I'll start this tomorrow morning." }
```

**Response `201 Created`:**
```json
{
  "id": "c9d8e7f6-...",
  "task_id": "t1a2b3c4-...",
  "user_id": "550e8400-...",
  "content": "I'll start this tomorrow morning.",
  "created_at": "2026-03-11T07:30:00Z",
  "updated_at": "2026-03-11T07:30:00Z"
}
```

---

#### `GET /tasks/{id}/comments`
List all comments for a task (oldest first).

**Response `200 OK`:** Array of Comment objects.

---

## Error Responses

All errors follow a consistent JSON format:

```json
{
  "code": 400,
  "error": "BAD_REQUEST",
  "message": "Detailed error description"
}
```

| HTTP Code | `error` Key | Description |
|-----------|-------------|-------------|
| `400` | `BAD_REQUEST` | Invalid request data |
| `401` | `UNAUTHORIZED` | Missing or invalid JWT token |
| `403` | `FORBIDDEN` | Insufficient permissions |
| `404` | `NOT_FOUND` | Resource does not exist |
| `409` | `CONFLICT` | Duplicate resource (e.g., email already exists) |
| `422` | `VALIDATION_ERROR` | Request body failed validation |
| `500` | `INTERNAL_SERVER_ERROR` | Server error (details hidden) |

### Security Notes

- Passwords are hashed with **Argon2id** (memory-hard, resistant to GPU attacks)
- Refresh tokens are stored as **SHA-256 hashes** — raw tokens never touch the database
- Login errors return a generic "Invalid email or password" message to prevent user enumeration
- Access tokens expire in **15 minutes** by default

---

## Health Check

```
GET /health
```

**Response `200 OK`:**
```json
{
  "status": "ok",
  "service": "calendar-task-api",
  "version": "1.0.0"
}
```
