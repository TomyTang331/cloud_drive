import React, { useEffect, useState } from 'react';
import Modal from '../common/Modal';
import { type FileItem } from '../../types';
import { formatBytes, formatFileType } from '../../utils/format';
import { fileService } from '../../services/api';
import './FileDetailsModal.less';

interface FileDetailsModalProps {
    isOpen: boolean;
    onClose: () => void;
    files: FileItem[];
}

// Helper to determine the icon based on file selection
const getFileIcon = (files: FileItem[]): string => {
    if (files.length === 1) {
        const file = files[0];
        if (file.file_type === 'folder') return '📁';
        if (file.mime_type?.startsWith('image/')) return '🖼️';
        if (file.mime_type?.startsWith('video/')) return '🎥';
        if (file.mime_type?.startsWith('audio/')) return '🎵';
        if (file.mime_type?.includes('pdf')) return '📕';
        return '📄';
    }

    const folderCount = files.filter(f => f.file_type === 'folder').length;
    const fileCount = files.filter(f => f.file_type === 'file').length;

    if (folderCount === files.length) return '📁';
    if (fileCount === files.length) return '📄';
    return '📚'; // Mixed selection
};

// Helper component for detail rows to reduce redundancy
const DetailRow: React.FC<{ label: string; value: React.ReactNode }> = ({ label, value }) => (
    <>
        <div className="details-label">{label}</div>
        <div className="details-value">{value}</div>
    </>
);

const FileDetailsModal: React.FC<FileDetailsModalProps> = ({ isOpen, onClose, files }) => {
    const [calculatedSize, setCalculatedSize] = useState<number | null>(null);
    const [isCalculating, setIsCalculating] = useState(false);

    useEffect(() => {
        if (!isOpen || !files || files.length === 0) {
            setCalculatedSize(null);
            setIsCalculating(false);
            return;
        }

        const calculate = async () => {
            const hasFolders = files.some(f => f.file_type === 'folder');

            if (hasFolders) {
                setIsCalculating(true);
                try {
                    const response = await fileService.calculateSize(files.map(f => f.id));
                    setCalculatedSize(response.data.data.total_size_bytes);
                } catch (error) {
                    console.error('Failed to calculate size:', error);
                    setCalculatedSize(null);
                } finally {
                    setIsCalculating(false);
                }
            } else {
                // For files only, sum up the size locally
                const total = files.reduce((acc, f) => acc + (f.size_bytes || 0), 0);
                setCalculatedSize(total);
                setIsCalculating(false);
            }
        };

        calculate();
    }, [isOpen, files]);

    if (!files || files.length === 0) return null;

    const isSingleFile = files.length === 1;
    const file = files[0];
    const folderCount = files.filter(f => f.file_type === 'folder').length;
    const fileCount = files.filter(f => f.file_type === 'file').length;

    // Determine type display text
    let typeDisplay = '';
    if (isSingleFile) {
        typeDisplay = formatFileType(file.mime_type, file.file_type === 'folder');
    } else if (folderCount === files.length) {
        typeDisplay = 'All folders';
    } else if (fileCount === files.length) {
        typeDisplay = 'All files';
    } else {
        typeDisplay = `Mixed (${folderCount} folders, ${fileCount} files)`;
    }

    // Determine location display text
    const locationDisplay = isSingleFile ? file.path : 'Multiple locations';

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title={isSingleFile ? "File Details" : `${files.length} Items Selected`}
            icon="ℹ️"
            footer={
                <button className="btn-create" onClick={onClose}>
                    Close
                </button>
            }
        >
            <div className="file-icon-large">
                {getFileIcon(files)}
            </div>

            <div className="details-grid">
                {isSingleFile ? (
                    <DetailRow label="Name:" value={file.name} />
                ) : (
                    <DetailRow label="Selected:" value={`${files.length} items`} />
                )}

                <DetailRow label="Type:" value={typeDisplay} />

                <DetailRow
                    label="Size:"
                    value={
                        isCalculating ? (
                            <span className="calculating-text">Calculating...</span>
                        ) : (
                            calculatedSize !== null ? formatBytes(calculatedSize) : '-'
                        )
                    }
                />

                <DetailRow label="Location:" value={locationDisplay} />

                {isSingleFile && (
                    <>
                        <DetailRow label="Created:" value={file.created_at} />
                        <DetailRow label="Modified:" value={file.updated_at} />
                    </>
                )}
            </div>

            {isSingleFile && (
                <div className="permissions-section">
                    <div className="permissions-title">Your Permissions</div>
                    <div className="permission-badges">
                        <span className={`permission-badge ${file.can_read ? 'granted' : ''}`}>
                            {file.can_read ? '✓' : '✕'} Read
                        </span>
                        <span className={`permission-badge ${file.can_write ? 'granted' : ''}`}>
                            {file.can_write ? '✓' : '✕'} Write
                        </span>
                        <span className={`permission-badge ${file.can_delete ? 'granted' : ''}`}>
                            {file.can_delete ? '✓' : '✕'} Delete
                        </span>
                    </div>
                </div>
            )}
        </Modal>
    );
};

export default FileDetailsModal;
