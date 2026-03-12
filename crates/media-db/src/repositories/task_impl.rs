impl TaskRepository for SqliteRepositories<'_> {
    fn exists(&self, task_id: &TaskId) -> Result<bool> {
        let exists = self
            .connection
            .query_row(
                "select 1 from tasks where id = :id limit 1",
                named_params! { ":id": task_id.0 },
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        Ok(exists)
    }

    fn upsert(&self, task: &TaskRecord) -> Result<()> {
        self.connection.execute(
            "
            insert into tasks (
              id, task_type, state, current, total, message, error_code,
              error_message, started_at, finished_at
            ) values (
              :id, :task_type, :state, :current, :total, :message, :error_code,
              :error_message, :started_at, :finished_at
            )
            on conflict(id) do update set
              task_type = excluded.task_type,
              state = excluded.state,
              current = excluded.current,
              total = excluded.total,
              message = excluded.message,
              error_code = excluded.error_code,
              error_message = excluded.error_message,
              started_at = excluded.started_at,
              finished_at = excluded.finished_at
            ",
            named_params! {
                ":id": task.id.0,
                ":task_type": task_kind_to_db(&task.task_type),
                ":state": task_state_to_db(&task.state),
                ":current": task.current,
                ":total": task.total,
                ":message": task.message,
                ":error_code": task.error_code,
                ":error_message": task.error_message,
                ":started_at": task.started_at,
                ":finished_at": task.finished_at,
            },
        )?;

        Ok(())
    }

    fn get(&self, task_id: &TaskId) -> Result<Option<TaskRecord>> {
        let record = self
            .connection
            .query_row(
                "
                select id, task_type, state, current, total, message, error_code,
                       error_message, started_at, finished_at
                from tasks
                where id = :id
                ",
                named_params! { ":id": task_id.0 },
                |row| {
                    Ok(TaskRecord {
                        id: TaskId(row.get::<_, String>(0)?),
                        task_type: task_kind_from_db(&row.get::<_, String>(1)?),
                        state: task_state_from_db(&row.get::<_, String>(2)?),
                        current: row.get::<_, u64>(3)?,
                        total: row.get(4)?,
                        message: row.get(5)?,
                        error_code: row.get(6)?,
                        error_message: row.get(7)?,
                        started_at: row.get(8)?,
                        finished_at: row.get(9)?,
                    })
                },
            )
            .optional()?;

        Ok(record)
    }
}
