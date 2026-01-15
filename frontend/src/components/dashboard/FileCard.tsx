import React from 'react';
import type { FileItem } from '../../types';
import { useThumbnail } from '../../hooks/useThumbnail';
import { formatBytes, formatFileType } from '../../utils/format';
import { getFileIcon } from '../../utils/fileUtils';
import './FileCard.less';
import DropdownMenu from '../common/DropdownMenu';

interface FileCardProps {
    file: FileItem;
    viewMode: 'grid' | 'list';
    selected: boolean;
    onSelect: (id: number, multi: boolean) => void;
    onClick: (file: FileItem) => void;
    onContextMenu: (e: React.MouseEvent, file: FileItem) => void;
    onDelete: (file: FileItem) => void;
}

const FileCard: React.FC<FileCardProps> = ({
    file,
    viewMode,
    selected,
    onSelect,
    onClick,
    onContextMenu,
    onDelete
}) => {
    const { thumbnailUrl } = useThumbnail(file);
    const [showMenu, setShowMenu] = React.useState(false);
    const menuButtonRef = React.useRef<HTMLButtonElement>(null);

    const handleMenuClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        setShowMenu(!showMenu);
    };

    return (
        <div
            className={`file-card ${viewMode} ${selected ? 'selected' : ''}`}
            onClick={(e) => {
                if (e.ctrlKey || e.metaKey || e.shiftKey) {
                    onSelect(file.id, true);
                } else {
                    onClick(file);
                }
            }}
            onContextMenu={(e) => {
                e.stopPropagation();
                onContextMenu(e, file);
            }}
        >
            <div className={`file-check ${selected ? 'visible' : ''}`} onClick={(e) => e.stopPropagation()}>
                <input
                    type="checkbox"
                    checked={selected}
                    onChange={() => onSelect(file.id, true)}
                />
            </div>

            {viewMode === 'grid' ? (
                // Grid View Content
                <div className="file-grid-content">
                    <div className="file-icon-wrapper">
                        {thumbnailUrl ? (
                            <img src={thumbnailUrl} alt={file.name} className="file-thumbnail" />
                        ) : (
                            <div className="file-icon-placeholder">
                                {file.file_type === 'folder' ? (
                                    <span className="folder-icon-yellow">📁</span>
                                ) : (
                                    getFileIcon(file)
                                )}
                            </div>
                        )}
                    </div>
                    <div className="file-name" title={file.name}>{file.name}</div>
                </div>
            ) : (
                // List View Content
                <>
                    <div className="file-icon-wrapper-list">
                        {file.file_type === 'folder' ? (
                            <span className="folder-icon-yellow-small">📁</span>
                        ) : (
                            thumbnailUrl ? <img src={thumbnailUrl} className="file-thumbnail-small" /> : getFileIcon(file)
                        )}
                    </div>

                    {/* Name Column Wrapper - Contains Name and Actions */}
                    <div className="file-name-col-wrapper">
                        <div className="file-col file-name-list" title={file.name}>
                            {file.name}
                        </div>

                        <div className="file-row-actions">
                            <button className="icon-btn" title="Share">🔗</button>
                            <button className="icon-btn" title="Download">⬇️</button>
                            <button
                                className="icon-btn"
                                title="More"
                                ref={menuButtonRef}
                                onClick={handleMenuClick}
                            >
                                ⋯
                            </button>
                        </div>
                    </div>

                    <div className="file-col file-size">
                        {file.file_type === 'folder' ? '-' : formatBytes(file.size_bytes)}
                    </div>
                    <div className="file-col file-type">
                        {file.file_type === 'folder' ? '文件夹' : formatFileType(file.mime_type, false)}
                    </div>
                    <div className="file-col file-date">
                        {new Date(file.created_at).toISOString().split('T')[0].replace(/-/g, '.')} {new Date(file.created_at).toTimeString().slice(0, 5)}
                    </div>
                </>
            )}

            <DropdownMenu
                isOpen={showMenu}
                onClose={() => setShowMenu(false)}
                position={menuButtonRef.current ? {
                    top: menuButtonRef.current.getBoundingClientRect().bottom + 5,
                    right: window.innerWidth - menuButtonRef.current.getBoundingClientRect().right
                } : undefined}
                items={[
                    { label: 'Move to', onClick: () => { /* TODO: Implement Move */ } },
                    { label: 'Copy to', onClick: () => { /* TODO: Implement Copy */ } },
                    { label: 'Rename', onClick: () => { /* TODO: Implement Rename */ } },
                    { label: 'Delete', onClick: () => onDelete(file), danger: true },
                    { label: 'Details', onClick: () => { /* TODO: Implement Details */ }, separator: true },
                ]}
            />
        </div>
    );
};

export default FileCard;
