import React, { useState, useEffect } from 'react';
import Modal from '../common/Modal';
import { fileService } from '../../services/api';
import type { FileItem } from '../../types';
import { IconFolder, IconFolderOpen } from '../common/Icons';
import { Colors } from '../../theme/colors';
import './FileSelectorModal.less';

interface FileSelectorModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSelect: (path: string) => void;
    title: string;
    actionLabel: string;
    excludePath?: string; // Path to exclude (e.g. moving a folder into itself)
}

const FileSelectorModal: React.FC<FileSelectorModalProps> = ({
    isOpen,
    onClose,
    onSelect,
    title,
    actionLabel,
    excludePath
}) => {
    const [currentPath, setCurrentPath] = useState('/');
    const [folders, setFolders] = useState<FileItem[]>([]);
    const [loading, setLoading] = useState(false);
    const [selectedPath, setSelectedPath] = useState<string | null>(null);

    useEffect(() => {
        if (isOpen) {
            setCurrentPath('/');
            setSelectedPath(null);
            loadFolders('/');
        }
    }, [isOpen]);

    const loadFolders = async (path: string) => {
        setLoading(true);
        try {
            const response = await fileService.listFiles(path);
            // Filter only folders
            const folderList = response.data.data.files.filter(f => f.file_type === 'folder');
            setFolders(folderList);
        } catch (error) {
            console.error('Failed to load folders:', error);
        } finally {
            setLoading(false);
        }
    };

    const handleNavigate = (path: string) => {
        setCurrentPath(path);
        setSelectedPath(null); // Deselect when navigating
        loadFolders(path);
    };

    const handleFolderClick = (folder: FileItem) => {
        // If single click, just select it as a potential destination
        setSelectedPath(folder.path);
    };

    const handleFolderDoubleClick = (folder: FileItem) => {
        // Enter folder
        handleNavigate(folder.path);
    };

    const handleBack = () => {
        if (currentPath === '/') return;
        const parentPath = currentPath.substring(0, currentPath.lastIndexOf('/')) || '/';
        handleNavigate(parentPath);
    };

    const handleConfirm = () => {
        // If a folder is selected, use that. Otherwise use current path.
        const targetPath = selectedPath || currentPath;

        // Basic validation to prevent moving into itself (though backend also checks)
        if (excludePath && targetPath.startsWith(excludePath)) {
            // Simple check, backend does robust check
            return;
        }

        onSelect(targetPath);
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title={title}
            icon="📂"
            footer={
                <>
                    <button className="btn-cancel" onClick={onClose}>
                        Cancel
                    </button>
                    <button
                        className="btn-create"
                        onClick={handleConfirm}
                        disabled={loading}
                    >
                        {actionLabel}
                    </button>
                </>
            }
            className="file-selector-modal"
        >
            <div className="file-selector-container">
                <div className="selector-header">
                    <button
                        className="btn-back"
                        onClick={handleBack}
                        disabled={currentPath === '/'}
                    >
                        ⬆️ Up
                    </button>
                    <div className="current-path">
                        {currentPath}
                    </div>
                </div>

                <div className="folder-list">
                    {loading ? (
                        <div className="loading">Loading...</div>
                    ) : (
                        <>
                            {folders.length === 0 ? (
                                <div className="empty-folder">No subfolders</div>
                            ) : (
                                folders.map(folder => {
                                    const isSelected = selectedPath === folder.path;
                                    const isExcluded = excludePath && (folder.path === excludePath || folder.path.startsWith(excludePath + '/'));

                                    return (
                                        <div
                                            key={folder.id}
                                            className={`folder-item ${isSelected ? 'selected' : ''} ${isExcluded ? 'disabled' : ''}`}
                                            onClick={() => !isExcluded && handleFolderClick(folder)}
                                            onDoubleClick={() => !isExcluded && handleFolderDoubleClick(folder)}
                                        >
                                            <div className="folder-icon">
                                                {isSelected ? <IconFolderOpen color={Colors.folder} /> : <IconFolder color={Colors.folder} />}
                                            </div>
                                            <span className="folder-name">{folder.name}</span>
                                        </div>
                                    );
                                })
                            )}
                        </>
                    )}
                </div>

                <div className="selector-footer-info">
                    Selected: {selectedPath ? selectedPath.split('/').pop() : 'Current Folder'}
                </div>
            </div>
        </Modal>
    );
};

export default FileSelectorModal;
