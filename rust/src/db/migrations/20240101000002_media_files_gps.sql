-- Add GPS coordinates to media_files (sensitive fields, not exposed via default API).
-- Existing rows keep NULL (no GPS known). Populated on next rescan via extract_exif().
ALTER TABLE media_files ADD COLUMN gps_latitude REAL;
ALTER TABLE media_files ADD COLUMN gps_longitude REAL;
