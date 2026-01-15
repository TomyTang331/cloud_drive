import type { FileItem } from '../types';

export const getFileIcon = (file: FileItem) => {
    if (file.file_type === 'folder') return '📁';

    if (!file.mime_type) return '📄';

    if (file.mime_type.startsWith('image/')) return '🖼️';
    if (file.mime_type.startsWith('video/')) return '🎬';
    if (file.mime_type.includes('pdf') || file.mime_type.includes('document')) return '📄';
    if (file.mime_type.includes('zip') || file.mime_type.includes('compressed')) return '📦';

    return '📄';
};
