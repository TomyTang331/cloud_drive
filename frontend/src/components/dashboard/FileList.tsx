import React from 'react';
import type { FileItem } from '../../types';
import Breadcrumbs from './Breadcrumbs';
import FileCard from './FileCard';

interface FileListProps {
    files: FileItem[];
    viewMode: 'grid' | 'list';
    loading: boolean;
    selectedIds: Set<number>;
    currentPath: string;
    onNavigate: (path: string) => void;
    onSelect: (id: number, multi: boolean) => void;
    onSelectAll: (select: boolean) => void;
    onFileClick: (file: FileItem) => void;
    onContextMenu: (e: React.MouseEvent, file?: FileItem) => void;
    onDelete: (file: FileItem) => void;
    onFileUpload: (files: FileList) => void;
}

const FileList: React.FC<FileListProps> = ({
    files,
    viewMode,
    loading,
    selectedIds,
    currentPath,
    onNavigate,
    onSelect,
    onSelectAll,
    onFileClick,
    onContextMenu,
    onDelete,
    onFileUpload
}) => {
    if (loading) {
        return (
            <div className="loading-state">
                <div className="spinner"></div>
                <p>Loading files...</p>
            </div>
        );
    }

    if (files.length === 0 && currentPath === '/') {
        return (
            <div className="empty-state">
                <div className="empty-icon">📂</div>
                <h3>No files yet</h3>
                <p>Drag and drop files here or use the New button to upload</p>
                <label className="btn-upload-empty">
                    Upload File
                    <input
                        type="file"
                        multiple
                        onChange={(e) => e.target.files && onFileUpload(e.target.files)}
                        style={{ display: 'none' }}
                    />
                </label>
            </div>
        );
    }

    return (
        <div
            className={`file-list-wrapper ${viewMode}`}
            onContextMenu={(e) => onContextMenu(e)}
        >
            <div className="file-list-header-container">
                {/* Breadcrumb Navigation with selection info */}
                <div className="breadcrumb-nav">
                    <Breadcrumbs currentPath={currentPath} onNavigate={onNavigate} />
                    {selectedIds.size > 0 && (
                        <div className="selection-badge">
                            <span className="selection-count">{selectedIds.size}</span>
                            <span className="selection-label">{selectedIds.size === 1 ? 'item selected' : 'items selected'}</span>
                        </div>
                    )}
                </div>

                {/* Column Headers - always visible */}
                <div className="list-header-main">
                    <div className="header-check">
                        <input
                            type="checkbox"
                            checked={files.length > 0 && selectedIds.size === files.length}
                            onChange={(e) => onSelectAll(e.target.checked)}
                        />
                    </div>

                    {viewMode === 'list' ? (
                        <>
                            <div className="header-col header-name">
                                <span>File</span>
                                <span className="header-file-count">All loaded, {files.length} items</span>
                            </div>
                            <div className="header-col header-size">Size</div>
                            <div className="header-col header-type">Type</div>
                            <div className="header-col header-date">Date Modified</div>
                        </>
                    ) : (
                        <div className="header-info">
                            <span className="header-title">Files</span>
                            <span className="header-count">{files.length} items</span>
                        </div>
                    )}
                </div>
            </div>

            <div className={`files-container ${viewMode}`}>
                {files.map((file) => (
                    <FileCard
                        key={file.id}
                        file={file}
                        viewMode={viewMode}
                        selected={selectedIds.has(file.id)}
                        onSelect={onSelect}
                        onClick={onFileClick}
                        onContextMenu={onContextMenu}
                        onDelete={onDelete}
                    />
                ))}
            </div>
        </div>
    );
};

export default FileList;
