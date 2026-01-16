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
        moveFile,
        copyFile,
        deleteFile,
        batchDeleteFiles,
        uploadFiles,
        batchDownloadFiles,
        setCurrentPath
    } = useFileOperations();

    const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
    const fileInputRef = useRef<HTMLInputElement>(null);
    // Ref to track if we are waiting for URL params to update
    const pendingUpdateRef = useRef(false);

    // File Details State
    const [detailsFiles, setDetailsFiles] = useState<FileItem[]>([]);

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
        showMoveModal,
        itemToMove,
        openMoveModal,
        closeMoveModal,
        showCopyModal,
        itemToCopy,
        openCopyModal,
        closeCopyModal,
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

    const handleContextMenu = (e: React.MouseEvent, file?: FileItem) => {
        e.preventDefault();
        setContextMenu({
            x: e.clientX,
            y: e.clientY,
            type: file ? 'file' : 'empty',
            file,
        });
    };

    useEffect(() => {
        const handleClick = () => {
            setContextMenu(null);
        };
        document.addEventListener('click', handleClick);
        return () => document.removeEventListener('click', handleClick);
    }, [setContextMenu]);

    const previewImages = React.useMemo(() => {
        return processedFiles.filter(f => f.mime_type?.startsWith('image/'));
    }, [processedFiles]);

    const handleCreateFolder = async () => {
        const success = await createFolder(newFolderName);
        if (success) {
            setNewFolderName('');
            setShowCreateFolder(false);
        }
    };

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

    const handleRename = async () => {
        if (!renameFileState) return;
        const success = await renameFile(renameFileState.id, newName);
        if (success) {
            closeRenameModal();
        }
    };

    const handleMove = async (destinationPath: string) => {
        if (!itemToMove) return;
        const success = await moveFile(itemToMove.id, destinationPath);
        if (success) {
            closeMoveModal();
        }
    };

    const handleCopy = async (destinationPath: string) => {
        if (!itemToCopy) return;
        const success = await copyFile(itemToCopy.id, destinationPath);
        if (success) {
            closeCopyModal();
        }
    };

    const handleDetails = (file?: FileItem) => {
        if (file) {
            // If the file is part of the current selection and we have multiple items selected,
            // show details for all selected items.
            if (selectedIds.has(file.id) && selectedIds.size > 1) {
                const selectedFiles = processedFiles.filter(f => selectedIds.has(f.id));
                setDetailsFiles(selectedFiles);
            } else {
                setDetailsFiles([file]);
            }
        } else if (selectedIds.size > 0) {
            const selectedFiles = processedFiles.filter(f => selectedIds.has(f.id));
            setDetailsFiles(selectedFiles);
        }
    };

    const handleDownload = (file: FileItem) => {
        if (selectedIds.has(file.id) && selectedIds.size > 1) {
            batchDownloadFiles(Array.from(selectedIds));
        } else {
            batchDownloadFiles([file.id]);
        }
    };

    const handleDelete = (file: FileItem) => {
        if (!file.can_delete) {
            toast.error('You don\'t have permission to delete this file');
            return;
        }

        if (selectedIds.has(file.id) && selectedIds.size > 1) {
            openDeleteModal();
        } else {
            openDeleteModal(file);
        }
    };

    const confirmDelete = async () => {
        if (itemToDelete) {
            const success = await deleteFile(itemToDelete.id);
            if (success) {
                closeDeleteModal();
                if (selectedIds.has(itemToDelete.id)) {
                    handleSelect(itemToDelete.id, true);
                }
            }
        } else if (selectedIds.size > 0) {
            const success = await batchDeleteFiles(Array.from(selectedIds));
            if (success) {
                closeDeleteModal();
                clearSelection();
            }
        }
    };

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
                            openDeleteModal();
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
                    onDetails={() => handleDetails()}
                    onMove={() => {
                        if (selectedIds.size === 1) {
                            const file = processedFiles.find(f => selectedIds.has(f.id));
                            if (file) openMoveModal(file);
                        } else {
                            toast.info('Batch move coming soon');
                        }
                    }}
                    onCopy={() => {
                        if (selectedIds.size === 1) {
                            const file = processedFiles.find(f => selectedIds.has(f.id));
                            if (file) openCopyModal(file);
                        } else {
                            toast.info('Batch copy coming soon');
                        }
                    }}
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
                    onRename={openRenameModal}
                    onMove={openMoveModal}
                    onCopy={openCopyModal}
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
                        onMove={openMoveModal}
                        onCopy={openCopyModal}
                        onDelete={handleDelete}
                        onDetails={handleDetails}
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
                    showMoveModal={showMoveModal}
                    closeMoveModal={closeMoveModal}
                    itemToMove={itemToMove}
                    onMove={handleMove}
                    showCopyModal={showCopyModal}
                    closeCopyModal={closeCopyModal}
                    itemToCopy={itemToCopy}
                    onCopy={handleCopy}
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
                    detailsFiles={detailsFiles}
                    setDetailsFiles={setDetailsFiles}
                    selectedCount={selectedIds.size}
                    currentPath={currentPath}
                />
            </main>
        </div>
    );
};

export default Dashboard;
