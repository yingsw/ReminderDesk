using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Microsoft.Data.Sqlite;
using ReminderDesk.Models;

namespace ReminderDesk.Services
{
    /// <summary>
    /// SQLite 数据库服务
    /// </summary>
    public class DatabaseService : IDisposable
    {
        private readonly string _connectionString;
        private readonly string _dbPath;

        public DatabaseService()
        {
            var appDataPath = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
            var appFolder = Path.Combine(appDataPath, "ReminderDesk");
            
            if (!Directory.Exists(appFolder))
            {
                Directory.CreateDirectory(appFolder);
            }

            _dbPath = Path.Combine(appFolder, "reminders.db");
            _connectionString = $"Data Source={_dbPath}";

            InitializeDatabase();
        }

        private void InitializeDatabase()
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            var sql = @"
                CREATE TABLE IF NOT EXISTS Reminders (
                    Id TEXT PRIMARY KEY,
                    Description TEXT NOT NULL,
                    Priority INTEGER DEFAULT 1,
                    DueTime TEXT NOT NULL,
                    ReminderTime TEXT NOT NULL,
                    IsCompleted INTEGER DEFAULT 0,
                    HasReminded INTEGER DEFAULT 0,
                    CreatedTime TEXT NOT NULL,
                    ReminderType INTEGER DEFAULT 0,
                    ReminderFunction TEXT
                );
                
                CREATE INDEX IF NOT EXISTS idx_due_time ON Reminders(DueTime);
                CREATE INDEX IF NOT EXISTS idx_reminder_time ON Reminders(ReminderTime);
            ";

            using var command = new SqliteCommand(sql, connection);
            command.ExecuteNonQuery();
        }

        public void SaveReminder(ReminderItem item)
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            var sql = @"
                INSERT OR REPLACE INTO Reminders 
                (Id, Description, Priority, DueTime, ReminderTime, IsCompleted, HasReminded, CreatedTime, ReminderType, ReminderFunction)
                VALUES (@Id, @Description, @Priority, @DueTime, @ReminderTime, @IsCompleted, @HasReminded, @CreatedTime, @ReminderType, @ReminderFunction)
            ";

            using var command = new SqliteCommand(sql, connection);
            command.Parameters.AddWithValue("@Id", item.Id.ToString());
            command.Parameters.AddWithValue("@Description", item.Description ?? "");
            command.Parameters.AddWithValue("@Priority", item.Priority);
            command.Parameters.AddWithValue("@DueTime", item.DueTime.ToString("o"));
            command.Parameters.AddWithValue("@ReminderTime", item.ReminderTime.ToString("o"));
            command.Parameters.AddWithValue("@IsCompleted", item.IsCompleted ? 1 : 0);
            command.Parameters.AddWithValue("@HasReminded", item.HasReminded ? 1 : 0);
            command.Parameters.AddWithValue("@CreatedTime", item.CreatedTime.ToString("o"));
            command.Parameters.AddWithValue("@ReminderType", item.ReminderType);
            command.Parameters.AddWithValue("@ReminderFunction", item.ReminderFunction ?? "");

            command.ExecuteNonQuery();
        }

        public List<ReminderItem> LoadAllReminders()
        {
            var list = new List<ReminderItem>();

            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            var sql = "SELECT * FROM Reminders ORDER BY DueTime";
            using var command = new SqliteCommand(sql, connection);
            using var reader = command.ExecuteReader();

            while (reader.Read())
            {
                list.Add(new ReminderItem
                {
                    Id = Guid.Parse(reader.GetString(0)),
                    Description = reader.GetString(1),
                    Priority = reader.GetInt32(2),
                    DueTime = DateTime.Parse(reader.GetString(3)),
                    ReminderTime = DateTime.Parse(reader.GetString(4)),
                    IsCompleted = reader.GetInt32(5) == 1,
                    HasReminded = reader.GetInt32(6) == 1,
                    CreatedTime = DateTime.Parse(reader.GetString(7)),
                    ReminderType = reader.GetInt32(8),
                    ReminderFunction = reader.IsDBNull(9) ? "" : reader.GetString(9)
                });
            }

            return list;
        }

        public void DeleteReminder(Guid id)
        {
            using var connection = new SqliteConnection(_connectionString);
            connection.Open();

            var sql = "DELETE FROM Reminders WHERE Id = @Id";
            using var command = new SqliteCommand(sql, connection);
            command.Parameters.AddWithValue("@Id", id.ToString());
            command.ExecuteNonQuery();
        }

        public void Dispose()
        {
        }
    }
}