---
name: tauri-command
description: Assists in implementing Tauri commands. Used for specific Tauri implementation patterns such as communication between frontend and backend, implementation of invoke functions, automatic conversion from camelCase to snake_case, and unified return value patterns.
model: sonnet
---

# Tauri Command Implementation Skill

This skill assists in implementing Tauri commands for the Flequit project.

## Important: Automatic Naming Convention Conversion

Tauri automatically converts JavaScript `camelCase` to Rust `snake_case`.

### Parameter Naming Rules

**JavaScript Side (camelCase)** ⇔ **Rust Side (snake_case)**

```typescript
// JavaScript/TypeScript - Use camelCase
await invoke('update_task', {
  projectId: 'project-123',        // Rust side: project_id
  taskId: 'task-456',             // Rust side: task_id
  partialSettings: {...}          // Rust side: partial_settings
});

await invoke('create_task_assignment', {
  taskAssignment: {               // Rust side: task_assignment
    task_id: 'task-123',
    user_id: 'user-456'
  }
});
```

```rust
// Rust side - Use snake_case
#[tauri::command]
pub async fn update_task(
    project_id: String,           // JavaScript side: projectId
    task_id: String,              // JavaScript side: taskId
    partial_settings: PartialSettings // JavaScript side: partialSettings
) -> Result<bool, String> {
    // Implementation
}

#[tauri::command]
pub async fn create_task_assignment(
    task_assignment: TaskAssignment  // JavaScript side: taskAssignment
) -> Result<bool, String> {
    // Implementation
}
```

## Unifying Return Values

### Return Values for Void Commands

```rust
// Rust side - Return () (Unit type) on success
#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<(), String> {
    // Save process
    Ok(()) // Return Unit type
}
```

```typescript
// JavaScript side - Treat success as true
async saveSettings(settings: Settings): Promise<boolean> {
  try {
    await invoke('save_settings', { settings });
    return true; // void success = true
  } catch (error) {
    console.error('Failed to save settings:', error);
    return false; // error = false
  }
}
```

## Unified Error Handling

### TypeScript Side Patterns

```typescript
// Generic error handling pattern
async function tauriServiceMethod<T>(
  command: string,
  params?: object
): Promise<T | null> {
  try {
    const result = (await invoke(command, params)) as T;
    return result;
  } catch (error) {
    console.error(`Failed to execute ${command}:`, error);
    return null;
  }
}

// For boolean return values
async function tauriBooleanMethod(
  command: string,
  params?: object
): Promise<boolean> {
  try {
    await invoke(command, params);
    return true;
  } catch (error) {
    console.error(`Failed to execute ${command}:`, error);
    return false;
  }
}
```

### Rust Side Patterns

```rust
#[tauri::command]
pub async fn get_task(
    project_id: String,
    task_id: String
) -> Result<Task, String> {
    task_facade::get_task(&repositories, &project_id, &task_id)
        .await
        .map_err(|e| format!("Failed to get task: {:?}", e))
}

#[tauri::command]
pub async fn delete_task(
    project_id: String,
    task_id: String
) -> Result<bool, String> {
    task_facade::delete_task(&repositories, &project_id, &task_id)
        .await
        .map_err(|e| format!("Failed to delete task: {:?}", e))
}
```

## Implementation Checklist

### JavaScript/TypeScript Implementation

- [ ] Write parameter names in `camelCase`
- [ ] Ensure correspondence with Rust `snake_case` function parameters
- [ ] Return `true`/`false` for void return value commands
- [ ] Implement appropriate error handling
- [ ] Output error details to the console log

### Rust Implementation

- [ ] Write function parameters in `snake_case`
- [ ] Ensure correspondence with JavaScript `camelCase` parameters
- [ ] Handle errors using `Result<T, String>`
- [ ] Provide appropriate error messages
- [ ] Add the `#[tauri::command]` attribute

## Complete Implementation Example

### TypeScript Side (Service)

```typescript
// src/lib/services/task-service.ts
import { invoke } from "@tauri-apps/api/tauri";
import type { Task, CreateTaskRequest } from "$lib/types";

export class TaskService {
  async getTasks(projectId: string): Promise<Task[] | null> {
    try {
      const tasks = await invoke<Task[]>("get_tasks", { projectId });
      return tasks;
    } catch (error) {
      console.error("Failed to get tasks:", error);
      return null;
    }
  }

  async createTask(request: CreateTaskRequest): Promise<Task | null> {
    try {
      const task = await invoke<Task>("create_task", {
        projectId: request.projectId,
        title: request.title,
        description: request.description,
      });
      return task;
    } catch (error) {
      console.error("Failed to create task:", error);
      return null;
    }
  }

  async updateTask(
    projectId: string,
    taskId: string,
    updates: Partial<Task>
  ): Promise<boolean> {
    try {
      await invoke("update_task", {
        projectId,
        taskId,
        partialTask: updates,
      });
      return true;
    } catch (error) {
      console.error("Failed to update task:", error);
      return false;
    }
  }

  async deleteTask(projectId: string, taskId: string): Promise<boolean> {
    try {
      const result = await invoke<boolean>("delete_task", {
        projectId,
        taskId,
      });
      return result;
    } catch (error) {
      console.error("Failed to delete task:", error);
      return false;
    }
  }
}

export const taskService = new TaskService();
```

### Rust Side (Command)

```rust
// src-tauri/src/commands/task.rs
use flequit_core::facades::task as task_facade;
use flequit_storage::models::command::task::{Task, PartialTask};

#[tauri::command]
pub async fn get_tasks(
    project_id: String,  // JavaScript: projectId
    state: tauri::State<'_, AppState>
) -> Result<Vec<Task>, String> {
    let repositories = &state.repositories;

    task_facade::get_tasks(repositories, &project_id)
        .await
        .map_err(|e| format!("Failed to get tasks: {:?}", e))
}

#[tauri::command]
pub async fn create_task(
    project_id: String,  // JavaScript: projectId
    title: String,
    description: Option<String>,
    state: tauri::State<'_, AppState>
) -> Result<Task, String> {
    let repositories = &state.repositories;

    task_facade::create_task(
        repositories,
        &project_id,
        &title,
        description.as_deref()
    )
    .await
    .map_err(|e| format!("Failed to create task: {:?}", e))
}

#[tauri::command]
pub async fn update_task(
    project_id: String,   // JavaScript: projectId
    task_id: String,      // JavaScript: taskId
    partial_task: PartialTask,  // JavaScript: partialTask
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let repositories = &state.repositories;

    task_facade::update_task(
        repositories,
        &project_id,
        &task_id,
        partial_task
    )
    .await
    .map_err(|e| format!("Failed to update task: {:?}", e))
}

#[tauri::command]
pub async fn delete_task(
    project_id: String,  // JavaScript: projectId
    task_id: String,     // JavaScript: taskId
    state: tauri::State<'_, AppState>
) -> Result<bool, String> {
    let repositories = &state.repositories;

    task_facade::delete_task(repositories, &project_id, &task_id)
        .await
        .map_err(|e| format!("Failed to delete task: {:?}", e))
}
```

### Command Registration

```rust
// src-tauri/src/lib.rs
use crate::commands::task;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            task::get_tasks,
            task::create_task,
            task::update_task,
            task::delete_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Important Notes

### 1. Scope of Automatic Conversion

Tauri's automatic conversion applies **only to parameter names**. It does NOT include:

- Struct field names
- Enum variant names

These must be handled manually.

### 2. Maintaining Consistency

Use the same pattern throughout the project:

- JavaScript: Always `camelCase`
- Rust: Always `snake_case`
- Match type definitions between TypeScript and Rust

### 3. Type Safety

Match TypeScript type definitions and Rust struct definitions:

```typescript
// TypeScript
interface Task {
  id: string;
  title: string;
  status: "todo" | "in_progress" | "completed";
}
```

```rust
// Rust
#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
}
```

### 4. Testing

Testing in an actual Tauri environment is recommended:

```bash
# Start in Tauri dev mode
bun run tauri dev

# Test command via browser developer tools
await window.__TAURI__.invoke('get_tasks', { projectId: 'test' });
```

## Related Documents

Refer to the following documents for details:

- `docs/en/develop/rules/coding-standards.md` - Tauri⇔Frontend communication rules
- `docs/en/develop/design/backend-tauri/rust-guidelines.md` - Rust design guidelines
- `docs/en/develop/design/error-handling.md` - Error handling
