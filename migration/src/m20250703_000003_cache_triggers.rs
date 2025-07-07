use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create function to refresh materialized views
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION refresh_group_caches()
                RETURNS TRIGGER AS $$
                BEGIN
                    REFRESH MATERIALIZED VIEW CONCURRENTLY group_hierarchy_cache;
                    REFRESH MATERIALIZED VIEW CONCURRENTLY group_financial_cache;
                    RETURN NULL;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION refresh_member_caches()
                RETURNS TRIGGER AS $$
                BEGIN
                    REFRESH MATERIALIZED VIEW CONCURRENTLY member_summary_cache;
                    RETURN NULL;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION refresh_financial_caches()
                RETURNS TRIGGER AS $$
                BEGIN
                    REFRESH MATERIALIZED VIEW CONCURRENTLY group_financial_cache;
                    REFRESH MATERIALIZED VIEW CONCURRENTLY member_summary_cache;
                    RETURN NULL;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Create audit trigger function
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION audit_trigger_function()
                RETURNS TRIGGER AS $$
                DECLARE
                    old_data JSONB;
                    new_data JSONB;
                BEGIN
                    -- Get old and new data as JSONB
                    IF TG_OP = 'DELETE' THEN
                        old_data := to_jsonb(OLD);
                        new_data := NULL;
                    ELSIF TG_OP = 'UPDATE' THEN
                        old_data := to_jsonb(OLD);
                        new_data := to_jsonb(NEW);
                    ELSIF TG_OP = 'INSERT' THEN
                        old_data := NULL;
                        new_data := to_jsonb(NEW);
                    END IF;

                    -- Insert audit record
                    INSERT INTO audit_logs (
                        table_name,
                        record_id,
                        operation,
                        old_values,
                        new_values,
                        changed_by,
                        changed_at,
                        ip_address,
                        user_agent
                    ) VALUES (
                        TG_TABLE_NAME,
                        CASE 
                            WHEN TG_OP = 'DELETE' THEN (old_data->>'id')::UUID
                            ELSE (new_data->>'id')::UUID
                        END,
                        TG_OP,
                        old_data,
                        new_data,
                        CASE 
                            WHEN TG_OP = 'DELETE' THEN (old_data->>'updated_by')::UUID
                            ELSE (new_data->>'updated_by')::UUID
                        END,
                        NOW(),
                        current_setting('request.ip_address', TRUE),
                        current_setting('request.user_agent', TRUE)
                    );

                    RETURN CASE 
                        WHEN TG_OP = 'DELETE' THEN OLD
                        ELSE NEW
                    END;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Create updated_at trigger function
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_updated_at_column()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Create triggers for groups table
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER groups_updated_at_trigger
                    BEFORE UPDATE ON groups
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER groups_audit_trigger
                    AFTER INSERT OR UPDATE OR DELETE ON groups
                    FOR EACH ROW
                    EXECUTE FUNCTION audit_trigger_function();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER groups_cache_refresh_trigger
                    AFTER INSERT OR UPDATE OR DELETE ON groups
                    FOR EACH STATEMENT
                    EXECUTE FUNCTION refresh_group_caches();
                "#,
            )
            .await?;

        // Create triggers for members table
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER members_updated_at_trigger
                    BEFORE UPDATE ON members
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER members_audit_trigger
                    AFTER INSERT OR UPDATE OR DELETE ON members
                    FOR EACH ROW
                    EXECUTE FUNCTION audit_trigger_function();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER members_cache_refresh_trigger
                    AFTER INSERT OR UPDATE OR DELETE ON members
                    FOR EACH STATEMENT
                    EXECUTE FUNCTION refresh_member_caches();
                "#,
            )
            .await?;

        // Create triggers for group_memberships table
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER group_memberships_updated_at_trigger
                    BEFORE UPDATE ON group_memberships
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER group_memberships_audit_trigger
                    AFTER INSERT OR UPDATE OR DELETE ON group_memberships
                    FOR EACH ROW
                    EXECUTE FUNCTION audit_trigger_function();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER group_memberships_cache_refresh_trigger
                    AFTER INSERT OR UPDATE OR DELETE ON group_memberships
                    FOR EACH STATEMENT
                    EXECUTE FUNCTION refresh_group_caches();
                "#,
            )
            .await?;

        // Create triggers for shares table
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER shares_updated_at_trigger
                    BEFORE UPDATE ON shares
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER shares_audit_trigger
                    AFTER INSERT OR UPDATE OR DELETE ON shares
                    FOR EACH ROW
                    EXECUTE FUNCTION audit_trigger_function();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER shares_cache_refresh_trigger
                    AFTER INSERT OR UPDATE OR DELETE ON shares
                    FOR EACH STATEMENT
                    EXECUTE FUNCTION refresh_financial_caches();
                "#,
            )
            .await?;

        // Create trigger to update share total_value when shares_owned or share_value changes
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_share_total_value()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.total_value = NEW.share_quantity * NEW.share_value;
                    NEW.last_transaction_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Create separate triggers for INSERT and UPDATE to avoid referencing OLD in INSERT trigger
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER shares_calculate_total_insert_trigger
                    BEFORE INSERT ON shares
                    FOR EACH ROW
                    EXECUTE FUNCTION update_share_total_value();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER shares_calculate_total_update_trigger
                    BEFORE UPDATE ON shares
                    FOR EACH ROW
                    WHEN (NEW.share_quantity IS DISTINCT FROM OLD.share_quantity OR NEW.share_value IS DISTINCT FROM OLD.share_value)
                    EXECUTE FUNCTION update_share_total_value();
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop triggers
        let triggers = vec![
            ("shares_calculate_total_insert_trigger", "shares"),
            ("shares_calculate_total_update_trigger", "shares"),
            ("shares_cache_refresh_trigger", "shares"),
            ("shares_audit_trigger", "shares"),
            ("shares_updated_at_trigger", "shares"),
            (
                "group_memberships_cache_refresh_trigger",
                "group_memberships",
            ),
            ("group_memberships_audit_trigger", "group_memberships"),
            ("group_memberships_updated_at_trigger", "group_memberships"),
            ("members_cache_refresh_trigger", "members"),
            ("members_audit_trigger", "members"),
            ("members_updated_at_trigger", "members"),
            ("groups_cache_refresh_trigger", "groups"),
            ("groups_audit_trigger", "groups"),
            ("groups_updated_at_trigger", "groups"),
        ];

        for (trigger_name, table_name) in triggers {
            manager
                .get_connection()
                .execute_unprepared(&format!(
                    "DROP TRIGGER IF EXISTS {} ON {};",
                    trigger_name, table_name
                ))
                .await
                .ok(); // Ignore errors if trigger doesn't exist
        }

        // Drop functions
        let functions = vec![
            "update_share_total_value()",
            "update_updated_at_column()",
            "audit_trigger_function()",
            "refresh_financial_caches()",
            "refresh_member_caches()",
            "refresh_group_caches()",
        ];

        for function_name in functions {
            manager
                .get_connection()
                .execute_unprepared(&format!("DROP FUNCTION IF EXISTS {};", function_name))
                .await
                .ok(); // Ignore errors if function doesn't exist
        }

        Ok(())
    }
}
