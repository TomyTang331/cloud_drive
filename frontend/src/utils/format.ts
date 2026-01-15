export const formatBytes = (bytes?: number, decimals = 1) => {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
};

export const formatFileType = (mimeType?: string, isFolder?: boolean): string => {
    if (isFolder) return 'Folder';
    if (!mimeType) return 'Unknown File';

    if (mimeType.startsWith('image/')) return 'Image';
    if (mimeType.startsWith('video/')) return 'Video';
    if (mimeType.startsWith('audio/')) return 'Audio';
    if (mimeType.includes('pdf')) return 'PDF File';
    if (mimeType.includes('word') || mimeType.includes('document')) return 'Word Document';
    if (mimeType.includes('sheet') || mimeType.includes('excel')) return 'Excel Spreadsheet';
    if (mimeType.includes('presentation') || mimeType.includes('powerpoint')) return 'PowerPoint';
    if (mimeType.includes('zip') || mimeType.includes('compressed')) return 'Archive';
    if (mimeType.includes('text')) return 'Text File';

    // Fallback: use the subtype (e.g., "application/json" -> "json")
    const parts = mimeType.split('/');
    if (parts.length > 1) return parts[1].toUpperCase() + ' File';

    return 'File';
};
