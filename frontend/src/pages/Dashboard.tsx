import React, { useState, useEffect, useRef } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import type { FileItem } from '../types';
import DashboardModals from '../components/dashboard/DashboardModals';
import Sidebar from '../components/layout/Sidebar';
import TopNav from '../components/layout/TopNav';
import ActionToolbar from '../components/dashboard/ActionToolbar';
import FileList from '../components/dashboard/FileList';
import ContextMenu from '../components/dashboard/ContextMenu';
import { useFileOperations } from '../hooks/useFileOperations';
import { useFileDragDrop } from '../hooks/useFileDragDrop';
import { useToast } from '../hooks/useToast';
import { useSelection } from '../hooks/useSelection';
import { useFileFilter } from '../hooks/useFileFilter';
import { useDashboardModals } from '../hooks/useDashboardModals';
import './Dashboard.less';

const Dashboard: React.FC = () => {
    const { user, logout } = useAuth();
    const navigate = useNavigate();
    const [searchParams, setSearchParams] = useSearchParams();
    const toast = useToast();

    // File Operations Hook
    const {
        files,
        currentPath,
        loading,
        storageInfo,
        createFolder,
        renameFile,
        deleteFile,
        uploadFiles,
        batchDownloadFiles,
        setCurrentPath
    } = useFileOperations();

    const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
    const fileInputRef = useRef<HTMLInputElement>(null);
    // Ref to track if we are waiting for URL params to update
    const pendingUpdateRef = useRef(false);

    // File Details State
    const [detailsFile, setDetailsFile] = useState<FileItem | null>(null);

    // Custom Hooks
    const {
        selectedCategory,
        setSelectedCategory,
        searchQuery,
        setSearchQuery,
        sortBy,
        setSortBy,
        processedFiles
    } = useFileFilter(files);

    const {
        selectedIds,
        handleSelect,
        handleSelectAll,
        clearSelection
    } = useSelection(processedFiles.map(f => f.id));

    const {
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
    } = useDashboardModals();

    // Clear selection when filters change
    useEffect(() => {
        clearSelection();
    }, [selectedCategory, searchQuery, sortBy, currentPath, clearSelection]);

    // Handle context menu
    const handleContextMenu = (e: React.MouseEvent, file?: FileItem) => {
        e.preventDefault();
        setContextMenu({
            x: e.clientX,
            y: e.clientY,
            type: file ? 'file' : 'empty',
            file,
        });
    };

    // Close context menu on click
    useEffect(() => {
        const handleClick = () => {
            setContextMenu(null);
        };
        document.addEventListener('click', handleClick);
        return () => document.removeEventListener('click', handleClick);
    }, [setContextMenu]);

    // Memoize images for preview navigation
    const previewImages = React.useMemo(() => {
        return processedFiles.filter(f => f.mime_type?.startsWith('image/'));
    }, [processedFiles]);

    // Create folder
    const handleCreateFolder = async () => {
        const success = await createFolder(newFolderName);
        if (success) {
            setNewFolderName('');
            setShowCreateFolder(false);
        }
    };

    // Handle file/folder click
    const handleFileClick = (file: FileItem) => {
        if (file.file_type === 'folder') {
            setCurrentPath(file.path);
        } else if (file.mime_type?.startsWith('image/')) {
            setPreviewFile(file);
            pendingUpdateRef.current = true;
            setSearchParams((prev: URLSearchParams) => {
                const newParams = new URLSearchParams(prev);
                newParams.set('preview', String(file.id));
                return newParams;
            });
        } else if (file.mime_type?.startsWith('video/')) {
            setVideoFile(file);
            pendingUpdateRef.current = true;
            setSearchParams((prev: URLSearchParams) => {
                const newParams = new URLSearchParams(prev);
                newParams.set('video', String(file.id));
                return newParams;
            });
        } else {

        }
    };

    // Sync URL params with modal state (Handle Back Button / Deep Link)
    useEffect(() => {
        const previewId = searchParams.get('preview');
        const videoId = searchParams.get('video');

        // If we are pending an update (user just clicked), and the params haven't updated yet,
        // we should NOT close the modals.
        if (pendingUpdateRef.current) {
            // Check if the params have caught up to our local state
            const isPreviewSynced = previewFile ? String(previewFile.id) === previewId : !previewId;
            const isVideoSynced = videoFile ? String(videoFile.id) === videoId : !videoId;

            if (isPreviewSynced && isVideoSynced) {
                // Params match local state, we are synced!
                pendingUpdateRef.current = false;
            } else {
                // Params are still old (e.g. missing ID), but we have local state.
                // Do NOT clobber local state.
                return;
            }
        }

        // Sync Preview
        if (previewId) {
            if (!previewFile || String(previewFile.id) !== previewId) {
                const file = files.find(f => String(f.id) === previewId);
                if (file) setPreviewFile(file);
            }
        } else {
            if (previewFile) setPreviewFile(null);
        }

        // Sync Video
        if (videoId) {
            if (!videoFile || String(videoFile.id) !== videoId) {
                const file = files.find(f => String(f.id) === videoId);
                if (file) setVideoFile(file);
            }
        } else {
            if (videoFile) setVideoFile(null);
        }
    }, [searchParams, files, previewFile, videoFile, setPreviewFile, setVideoFile]);

    // Rename File
    const handleRename = async () => {
        if (!renameFileState) return;
        const success = await renameFile(renameFileState, newName);
        if (success) {
            closeRenameModal();
        }
    };

    // Download File
    const handleDownload = (file: FileItem) => {
        // If the file is part of the current selection and we have multiple items selected,
        // download all selected items.
        if (selectedIds.has(file.id) && selectedIds.size > 1) {
            batchDownloadFiles(Array.from(selectedIds));
        } else {
            // Otherwise just download the single file (even if others are selected, 
            // usually right-clicking an unselected item implies acting on just that one,
            // or the selection logic should have updated. But here we play it safe).
            batchDownloadFiles([file.id]);
        }
    };

    // Delete file/folder - Show confirmation
    const handleDelete = (file: FileItem) => {
        if (!file.can_delete) {
            toast.error('You don\'t have permission to delete this file');
            return;
        }
        openDeleteModal(file);
    };

    // Confirm delete
    const confirmDelete = async () => {
        if (!itemToDelete) return;
        const success = await deleteFile(itemToDelete);
        if (success) {
            closeDeleteModal();
        }
    };

    // Handle File Upload
    const handleFileUpload = async (filesToUpload: FileList | null) => {
        if (!filesToUpload) return;
        await uploadFiles(filesToUpload);
    };

    const { isDragging, handleDragOver, handleDragLeave, handleDrop } = useFileDragDrop({
        onUpload: async (files) => await handleFileUpload(files)
    });

    const triggerFileUpload = () => {
        fileInputRef.current?.click();
    };

    const handleLogout = () => {
        logout();
        navigate('/login');
    };

    return (
        <div
            className="dashboard-container"
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
        >
            {/* Drag Overlay */}
            {isDragging && (
                <div className="drag-overlay">
                    <div className="drag-content">
                        <div className="drag-icon">☁️</div>
                        <h3>Drop files to upload</h3>
                    </div>
                </div>
            )}

            <Sidebar
                storageInfo={storageInfo}
                selectedCategory={selectedCategory}
                onSelectCategory={setSelectedCategory}
                onNewFolder={() => setShowCreateFolder(true)}
            />

            {/* Main Content */}
            <main className="main-content">
                <TopNav
                    user={user}
                    searchQuery={searchQuery}
                    onSearchChange={setSearchQuery}
                    onLogout={handleLogout}
                />

                <ActionToolbar
                    viewMode={viewMode}
                    onViewModeChange={setViewMode}
                    onUpload={triggerFileUpload}
                    onNewFolder={() => setShowCreateFolder(true)}
                    onDelete={() => {
                        if (selectedIds.size > 0) {
                            // TODO: Implement batch delete
                            const fileToDelete = processedFiles.find(f => selectedIds.has(f.id));
                            if (fileToDelete) handleDelete(fileToDelete);
                        }
                    }}
                    onDownload={() => {
                        if (selectedIds.size > 0) {
                            const idsToDownload = Array.from(selectedIds);
                            batchDownloadFiles(idsToDownload);
                        }
                    }}
                    selectedCount={selectedIds.size}
                    onRename={() => {
                        if (selectedIds.size === 1) {
                            const file = processedFiles.find(f => selectedIds.has(f.id));
                            if (file) openRenameModal(file);
                        }
                    }}
                    onDetails={() => {
                        if (selectedIds.size === 1) {
                            const file = processedFiles.find(f => selectedIds.has(f.id));
                            if (file) setDetailsFile(file);
                        }
                    }}
                    onMove={() => toast.info('Move feature coming soon')}
                    onCopy={() => toast.info('Copy feature coming soon')}
                />

                <FileList
                    files={processedFiles}
                    viewMode={viewMode}
                    loading={loading}
                    selectedIds={selectedIds}
                    currentPath={currentPath}
                    onNavigate={setCurrentPath}
                    onSelect={handleSelect}
                    onSelectAll={handleSelectAll}
                    onFileClick={handleFileClick}
                    onContextMenu={handleContextMenu}
                    onDelete={handleDelete}
                    onFileUpload={handleFileUpload}
                />

                {/* Context Menu */}
                {contextMenu && (
                    <ContextMenu
                        x={contextMenu.x}
                        y={contextMenu.y}
                        type={contextMenu.type}
                        file={contextMenu.file || null}
                        sortBy={sortBy}
                        onClose={() => setContextMenu(null)}
                        onNewFolder={() => setShowCreateFolder(true)}
                        onUploadClick={triggerFileUpload}
                        onSort={(sort) => setSortBy(sort)}
                        onOpen={handleFileClick}
                        onDownload={handleDownload}
                        onRename={openRenameModal}
                        onDelete={handleDelete}
                        onDetails={(file) => setDetailsFile(file)}
                    />
                )}

                {/* Hidden File Input */}
                <input
                    type="file"
                    ref={fileInputRef}
                    multiple
                    style={{ display: 'none' }}
                    onChange={(e) => {
                        if (e.target.files) handleFileUpload(e.target.files);
                        e.target.value = '';
                    }}
                />

                {/* Dashboard Modals */}
                <DashboardModals
                    showCreateFolder={showCreateFolder}
                    setShowCreateFolder={setShowCreateFolder}
                    newFolderName={newFolderName}
                    setNewFolderName={setNewFolderName}
                    onCreateFolder={handleCreateFolder}
                    showRenameModal={showRenameModal}
                    closeRenameModal={closeRenameModal}
                    newName={newName}
                    setNewName={setNewName}
                    onRename={handleRename}
                    showDeleteModal={showDeleteModal}
                    closeDeleteModal={closeDeleteModal}
                    itemToDelete={itemToDelete}
                    onDelete={confirmDelete}
                    previewFile={previewFile}
                    setPreviewFile={(file) => {
                        setPreviewFile(file);
                        if (!file) {
                            setSearchParams((prev: URLSearchParams) => {
                                const newParams = new URLSearchParams(prev);
                                newParams.delete('preview');
                                return newParams;
                            });
                        } else {
                            setSearchParams((prev: URLSearchParams) => {
                                const newParams = new URLSearchParams(prev);
                                newParams.set('preview', String(file.id));
                                return newParams;
                            });
                        }
                    }}
                    previewImages={previewImages}
                    onDeleteFile={handleDelete}
                    videoFile={videoFile}
                    setVideoFile={(file) => {
                        setVideoFile(file);
                        if (!file) {
                            setSearchParams((prev: URLSearchParams) => {
                                const newParams = new URLSearchParams(prev);
                                newParams.delete('video');
                                return newParams;
                            });
                        } else {
                            setSearchParams((prev: URLSearchParams) => {
                                const newParams = new URLSearchParams(prev);
                                newParams.set('video', String(file.id));
                                return newParams;
                            });
                        }
                    }}
                    detailsFile={detailsFile}
                    setDetailsFile={setDetailsFile}
                />
            </main >
        </div >
    );
};

export default Dashboard;
