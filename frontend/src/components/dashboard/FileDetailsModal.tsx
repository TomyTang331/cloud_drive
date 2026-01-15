import React from 'react';
import Modal from '../common/Modal';
import { type FileItem } from '../../types';
import { formatBytes, formatFileType } from '../../utils/format';
import './FileDetailsModal.less';

interface FileDetailsModalProps {
    isOpen: boolean;
    onClose: () => void;
    file: FileItem | null;
}

const FileDetailsModal: React.FC<FileDetailsModalProps> = ({ isOpen, onClose, file }) => {
    if (!file) return null;

    const isFolder = file.file_type === 'folder';

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="File Details"
            icon="ℹ️"
            footer={
                <button className="btn-create" onClick={onClose}>
                    Close
                </button>
            }
        >
            <div className="file-icon-large">
                {isFolder ? '📁' : '📄'}
            </div>

            <div className="details-grid">
                <div className="details-label">Name:</div>
                <div className="details-value">{file.name}</div>

                <div className="details-label">Type:</div>
                <div className="details-value">{formatFileType(file.mime_type, isFolder)}</div>

                <div className="details-label">Size:</div>
                <div className="details-value">
                    {isFolder ? '-' : formatBytes(file.size_bytes || 0)}
                </div>

                <div className="details-label">Location:</div>
                <div className="details-value">{file.path}</div>

                <div className="details-label">Created:</div>
                <div className="details-value">{file.created_at}</div>

                <div className="details-label">Modified:</div>
                <div className="details-value">{file.updated_at}</div>
            </div>

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
        </Modal>
    );
};

export default FileDetailsModal;
