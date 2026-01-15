import React from 'react';
import Modal from '../common/Modal';
import ImagePreviewModal from './ImagePreviewModal';
import VideoPlayerModal from './VideoPlayerModal';
import FileDetailsModal from './FileDetailsModal';
import type { FileItem } from '../../types';

interface DashboardModalsProps {
    // Create Folder
    showCreateFolder: boolean;
    setShowCreateFolder: (show: boolean) => void;
    newFolderName: string;
    setNewFolderName: (name: string) => void;
    onCreateFolder: () => void;

    // Rename
    showRenameModal: boolean;
    closeRenameModal: () => void;
    newName: string;
    setNewName: (name: string) => void;
    onRename: () => void;

    // Delete
    showDeleteModal: boolean;
    closeDeleteModal: () => void;
    itemToDelete: FileItem | null;
    onDelete: () => void;

    // Preview
    previewFile: FileItem | null;
    setPreviewFile: (file: FileItem | null) => void;
    previewImages: FileItem[];
    onDeleteFile: (file: FileItem) => void;

    // Video Player
    videoFile: FileItem | null;
    setVideoFile: (file: FileItem | null) => void;

    // File Details
    detailsFile: FileItem | null;
    setDetailsFile: (file: FileItem | null) => void;
}

const DashboardModals: React.FC<DashboardModalsProps> = ({
    showCreateFolder,
    setShowCreateFolder,
    newFolderName,
    setNewFolderName,
    onCreateFolder,
    showRenameModal,
    closeRenameModal,
    newName,
    setNewName,
    onRename,
    showDeleteModal,
    closeDeleteModal,
    itemToDelete,
    onDelete,
    previewFile,
    setPreviewFile,
    previewImages,
    onDeleteFile,
    videoFile,
    setVideoFile,
    detailsFile,
    setDetailsFile
}) => {
    return (
        <>
            {/* Create Folder Modal */}
            <Modal
                isOpen={showCreateFolder}
                onClose={() => setShowCreateFolder(false)}
                title="Create New Folder"
                icon="📁"
                footer={
                    <>
                        <button className="btn-cancel" onClick={() => setShowCreateFolder(false)}>
                            Cancel
                        </button>
                        <button className="btn-create" onClick={onCreateFolder}>
                            Create
                        </button>
                    </>
                }
            >
                <input
                    type="text"
                    placeholder="Folder Name"
                    value={newFolderName}
                    onChange={(e) => setNewFolderName(e.target.value)}
                    autoFocus
                    onKeyDown={(e) => e.key === 'Enter' && onCreateFolder()}
                />
            </Modal>

            {/* Rename Modal */}
            <Modal
                isOpen={showRenameModal}
                onClose={closeRenameModal}
                title="Rename Item"
                icon="✏️"
                footer={
                    <>
                        <button className="btn-cancel" onClick={closeRenameModal}>
                            Cancel
                        </button>
                        <button className="btn-create" onClick={onRename}>
                            Rename
                        </button>
                    </>
                }
            >
                <input
                    type="text"
                    placeholder="New Name"
                    value={newName}
                    onChange={(e) => setNewName(e.target.value)}
                    autoFocus
                    onKeyDown={(e) => e.key === 'Enter' && onRename()}
                />
            </Modal>

            {/* Delete Confirmation Modal */}
            <Modal
                isOpen={showDeleteModal}
                onClose={closeDeleteModal}
                title="Delete Item?"
                icon={<div className="delete-icon">🗑️</div>}
                footer={
                    <>
                        <button className="btn-cancel" onClick={closeDeleteModal}>
                            Cancel
                        </button>
                        <button className="btn-delete" onClick={onDelete}>
                            Delete
                        </button>
                    </>
                }
                className="delete-modal"
            >
                <p>
                    Are you sure you want to delete <strong>"{itemToDelete?.name}"</strong>?
                    <br />
                    This action cannot be undone.
                </p>
            </Modal>

            {/* Image Preview Modal */}
            <ImagePreviewModal
                isOpen={!!previewFile}
                onClose={() => setPreviewFile(null)}
                file={previewFile}
                onNext={() => {
                    const currentIndex = previewImages.findIndex(f => f.id === previewFile?.id);
                    if (currentIndex < previewImages.length - 1) {
                        setPreviewFile(previewImages[currentIndex + 1]);
                    }
                }}
                onPrev={() => {
                    const currentIndex = previewImages.findIndex(f => f.id === previewFile?.id);
                    if (currentIndex > 0) {
                        setPreviewFile(previewImages[currentIndex - 1]);
                    }
                }}
                hasNext={(() => {
                    const currentIndex = previewImages.findIndex(f => f.id === previewFile?.id);
                    return currentIndex < previewImages.length - 1;
                })()}
                hasPrev={(() => {
                    const currentIndex = previewImages.findIndex(f => f.id === previewFile?.id);
                    return currentIndex > 0;
                })()}
                onDelete={onDeleteFile}
            />

            {/* Video Player Modal */}
            <VideoPlayerModal
                isOpen={!!videoFile}
                onClose={() => setVideoFile(null)}
                file={videoFile}
                onDelete={onDeleteFile}
            />

            {/* File Details Modal */}
            <FileDetailsModal
                isOpen={!!detailsFile}
                onClose={() => setDetailsFile(null)}
                file={detailsFile}
            />
        </>
    );
};

export default DashboardModals;
