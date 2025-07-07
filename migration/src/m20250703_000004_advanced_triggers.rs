use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Update the existing trigger function to use the correct field name
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_share_total_value()
                RETURNS TRIGGER AS $$
                BEGIN
                    -- Calculate total_value as share_quantity * share_value
                    NEW.total_value = NEW.share_quantity * NEW.share_value;
                    
                    -- Update the last_transaction_at timestamp
                    NEW.last_transaction_at = NOW();
                    
                    -- Update the updated_at timestamp
                    NEW.updated_at = NOW();
                    
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Drop and recreate the existing triggers with correct field references
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS shares_calculate_total_update_trigger ON shares",
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

        // Create trigger function to update shares_remaining in share_offers table
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_shares_remaining()
                RETURNS TRIGGER AS $$
                BEGIN
                    -- Calculate shares_remaining as total_shares_available - shares_sold
                    NEW.shares_remaining = NEW.total_shares_available - NEW.shares_sold;
                    
                    -- Update the updated_at timestamp
                    NEW.updated_at = NOW();
                    
                    -- Auto-complete offer if all shares are sold
                    IF NEW.shares_remaining <= 0 AND NEW.status = 'active' THEN
                        NEW.status = 'completed';
                    END IF;
                    
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Create trigger on share_offers table for INSERT and UPDATE
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER share_offers_remaining_trigger
                    BEFORE INSERT OR UPDATE OF total_shares_available, shares_sold
                    ON share_offers
                    FOR EACH ROW
                    EXECUTE FUNCTION update_shares_remaining();
                "#,
            )
            .await?;

        // Create trigger function to automatically update shares_sold when shares are purchased
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_offer_shares_sold()
                RETURNS TRIGGER AS $$
                BEGIN
                    -- When a new share is inserted, update the corresponding offer's shares_sold
                    IF TG_OP = 'INSERT' THEN
                        UPDATE share_offers 
                        SET shares_sold = shares_sold + NEW.share_quantity,
                            updated_at = NOW()
                        WHERE id = NEW.share_offer_id;
                        
                        RETURN NEW;
                    END IF;
                    
                    -- When a share is updated, adjust the offer's shares_sold
                    IF TG_OP = 'UPDATE' THEN
                        UPDATE share_offers 
                        SET shares_sold = shares_sold - OLD.share_quantity + NEW.share_quantity,
                            updated_at = NOW()
                        WHERE id = NEW.share_offer_id;
                        
                        RETURN NEW;
                    END IF;
                    
                    -- When a share is deleted, reduce the offer's shares_sold
                    IF TG_OP = 'DELETE' THEN
                        UPDATE share_offers 
                        SET shares_sold = shares_sold - OLD.share_quantity,
                            updated_at = NOW()
                        WHERE id = OLD.share_offer_id;
                        
                        RETURN OLD;
                    END IF;
                    
                    RETURN NULL;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Create trigger on shares table to automatically update offer shares_sold
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER shares_update_offer_trigger
                    AFTER INSERT OR UPDATE OF share_quantity OR DELETE
                    ON shares
                    FOR EACH ROW
                    EXECUTE FUNCTION update_offer_shares_sold();
                "#,
            )
            .await?;

        // Create function to validate share purchases
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION validate_share_purchase()
                RETURNS TRIGGER AS $$
                DECLARE
                    offer_record RECORD;
                    current_sold DECIMAL;
                BEGIN
                    -- Get the offer details
                    SELECT * INTO offer_record 
                    FROM share_offers 
                    WHERE id = NEW.share_offer_id;
                    
                    -- Check if offer exists
                    IF NOT FOUND THEN
                        RAISE EXCEPTION 'Share offer with id % does not exist', NEW.share_offer_id;
                    END IF;
                    
                    -- Check if offer is active
                    IF offer_record.status != 'active' THEN
                        RAISE EXCEPTION 'Share offer % is not active (status: %)', NEW.share_offer_id, offer_record.status;
                    END IF;
                    
                    -- Check validity period
                    IF offer_record.valid_from IS NOT NULL AND NOW() < offer_record.valid_from THEN
                        RAISE EXCEPTION 'Share offer % is not yet valid (starts: %)', NEW.share_offer_id, offer_record.valid_from;
                    END IF;
                    
                    IF offer_record.valid_until IS NOT NULL AND NOW() > offer_record.valid_until THEN
                        RAISE EXCEPTION 'Share offer % has expired (ended: %)', NEW.share_offer_id, offer_record.valid_until;
                    END IF;
                    
                    -- Calculate current shares sold (including this transaction)
                    current_sold = offer_record.shares_sold;
                    IF TG_OP = 'INSERT' THEN
                        current_sold = current_sold + NEW.share_quantity;
                    ELSIF TG_OP = 'UPDATE' THEN
                        current_sold = current_sold - OLD.share_quantity + NEW.share_quantity;
                    END IF;
                    
                    -- Check if enough shares are available
                    IF current_sold > offer_record.total_shares_available THEN
                        RAISE EXCEPTION 'Insufficient shares available in offer %. Available: %, Requested: %', 
                            NEW.share_offer_id, 
                            (offer_record.total_shares_available - offer_record.shares_sold),
                            NEW.share_quantity;
                    END IF;
                    
                    -- Check minimum purchase quantity
                    IF offer_record.min_purchase_quantity IS NOT NULL AND NEW.share_quantity < offer_record.min_purchase_quantity THEN
                        RAISE EXCEPTION 'Purchase quantity % is below minimum required %', NEW.share_quantity, offer_record.min_purchase_quantity;
                    END IF;
                    
                    -- Check maximum purchase quantity
                    IF offer_record.max_purchase_quantity IS NOT NULL AND NEW.share_quantity > offer_record.max_purchase_quantity THEN
                        RAISE EXCEPTION 'Purchase quantity % exceeds maximum allowed %', NEW.share_quantity, offer_record.max_purchase_quantity;
                    END IF;
                    
                    -- Ensure share_value matches offer price
                    IF NEW.share_value != offer_record.price_per_share THEN
                        NEW.share_value = offer_record.price_per_share;
                    END IF;
                    
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Create validation trigger on shares table
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER shares_validation_trigger
                    BEFORE INSERT OR UPDATE
                    ON shares
                    FOR EACH ROW
                    EXECUTE FUNCTION validate_share_purchase();
                "#,
            )
            .await?;

        // Add triggers for share_offers table
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER share_offers_updated_at_trigger
                    BEFORE UPDATE ON share_offers
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER share_offers_audit_trigger
                    AFTER INSERT OR UPDATE OR DELETE ON share_offers
                    FOR EACH ROW
                    EXECUTE FUNCTION audit_trigger_function();
                "#,
            )
            .await?;

        // Add audit logs table updated_at trigger
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER audit_logs_updated_at_trigger
                    BEFORE UPDATE ON audit_logs
                    FOR EACH ROW
                    EXECUTE FUNCTION update_updated_at_column();
                "#,
            )
            .await?;

        // Create function to auto-expire share offers
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION auto_expire_offers()
                RETURNS VOID AS $$
                BEGIN
                    UPDATE share_offers 
                    SET status = 'expired', 
                        updated_at = NOW()
                    WHERE status = 'active' 
                      AND valid_until IS NOT NULL 
                      AND valid_until < NOW();
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Create function to validate owner exists
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION validate_owner_exists()
                RETURNS TRIGGER AS $$
                DECLARE
                    owner_exists BOOLEAN := FALSE;
                BEGIN
                    -- Check if owner exists based on owner_type
                    IF NEW.owner_type = 'member' THEN
                        SELECT EXISTS(SELECT 1 FROM members WHERE id = NEW.owner_id) INTO owner_exists;
                    ELSIF NEW.owner_type = 'group' THEN
                        SELECT EXISTS(SELECT 1 FROM groups WHERE id = NEW.owner_id) INTO owner_exists;
                    END IF;
                    
                    IF NOT owner_exists THEN
                        RAISE EXCEPTION 'Owner with id % and type % does not exist', NEW.owner_id, NEW.owner_type;
                    END IF;
                    
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                "#,
            )
            .await?;

        // Create trigger to validate owner exists
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TRIGGER shares_owner_validation_trigger
                    BEFORE INSERT OR UPDATE
                    ON shares
                    FOR EACH ROW
                    EXECUTE FUNCTION validate_owner_exists();
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop new triggers
        let triggers = vec![
            ("shares_owner_validation_trigger", "shares"),
            ("audit_logs_updated_at_trigger", "audit_logs"),
            ("share_offers_audit_trigger", "share_offers"),
            ("share_offers_updated_at_trigger", "share_offers"),
            ("shares_validation_trigger", "shares"),
            ("shares_update_offer_trigger", "shares"),
            ("share_offers_remaining_trigger", "share_offers"),
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

        // Drop new functions
        let functions = vec![
            "validate_owner_exists()",
            "auto_expire_offers()",
            "validate_share_purchase()",
            "update_offer_shares_sold()",
            "update_shares_remaining()",
        ];

        for function_name in functions {
            manager
                .get_connection()
                .execute_unprepared(&format!("DROP FUNCTION IF EXISTS {};", function_name))
                .await
                .ok(); // Ignore errors if function doesn't exist
        }

        // Recreate the old trigger with the old field reference
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
}
