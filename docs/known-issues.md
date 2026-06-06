# Known Issues

Known issues and workarounds for Latte Album.

## iPhone mov Video Seek Failure

**Problem**: iPhone mov videos fail to seek to middle/end positions.

**Root cause**: iPhone mov files have moov atom at file end (HEVC/H.265 common practice). Range requests return data without moov metadata.

**Solution**: Use full file request for mov format, or detect moov position server-side.

## Timezone Handling

**Design**: Frontend shows time literal without conversion. Backend sorts by literal time. Timezone label shown only when different from user's local timezone.

**Known issue**: Photos from different timezones may be out of order. Not currently fixed.
