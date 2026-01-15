import { useState } from 'react';
import type { FileItem } from '../types';

export const useDashboardModals = () => {
    // Create Folder Modal
    const [showCreateFolder, setShowCreateFolder] = useState(false);
    const [newFolderName, setNewFolderName] = useState('');

    // Rename Modal
    const [showRenameModal, setShowRenameModal] = useState(false);
    const [renameFileState, setRenameFileState] = useState<FileItem | null>(null);
    const [newName, setNewName] = useState('');

    // Delete Modal
    const [showDeleteModal, setShowDeleteModal] = useState(false);
    const [itemToDelete, setItemToDelete] = useState<FileItem | null>(null);

    // Preview Modal
    const [previewFile, setPreviewFile] = useState<FileItem | null>(null);

    // Context Menu
    const [contextMenu, setContextMenu] = useState<{
        x: number;
        y: number;
        type: 'empty' | 'file';
        file?: FileItem;
    } | null>(null);

    const openRenameModal = (file: FileItem) => {
        setRenameFileState(file);
        setNewName(file.name);
        setShowRenameModal(true);
    };

    const closeRenameModal = () => {
        setShowRenameModal(false);
        setRenameFileState(null);
        setNewName('');
    };

    const openDeleteModal = (file: FileItem) => {
        setItemToDelete(file);
        setShowDeleteModal(true);
    };

    const closeDeleteModal = () => {
        setShowDeleteModal(false);
        setItemToDelete(null);
    };

    // Video Player Modal
    const [videoFile, setVideoFile] = useState<FileItem | null>(null);

    return {
        showCreateFolder,
        setShowCreateFolder,
        newFolderName,
        setNewFolderName,
        showRenameModal,
        renameFileState,
        newName,
        setNewName,
        openRenameModal,
        closeRenameModal,
        showDeleteModal,
        itemToDelete,
        openDeleteModal,
        closeDeleteModal,
        previewFile,
        setPreviewFile,
        videoFile,
        setVideoFile,
        contextMenu,
        setContextMenu
    };
};
