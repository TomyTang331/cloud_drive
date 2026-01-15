import React, { useRef } from 'react';
import './ActionToolbar.less';
import MoreActionsMenu from './MoreActionsMenu';

interface ActionToolbarProps {
    viewMode: 'grid' | 'list';
    onViewModeChange: (mode: 'grid' | 'list') => void;
    onUpload: () => void;
    onNewFolder: () => void;
    onNewFile?: () => void;
    onDelete: () => void;
    onDownload: () => void;
    onFileUpload?: (files: FileList) => void;
    selectedCount: number;
    onRename?: () => void;
    onDetails?: () => void;
    onMove?: () => void;
    onCopy?: () => void;
}

const ActionToolbar: React.FC<ActionToolbarProps> = ({
    viewMode,
    onViewModeChange,
    onUpload,
    onNewFolder,
    onNewFile,
    onDelete,
    onDownload,
    onFileUpload,
    selectedCount,
    onRename,
    onDetails,
    onMove,
    onCopy
}) => {
    const fileInputRef = useRef<HTMLInputElement>(null);
    const [showMoreMenu, setShowMoreMenu] = React.useState(false);
    const [menuPosition, setMenuPosition] = React.useState({ x: 0, y: 0 });
    const moreButtonRef = useRef<HTMLButtonElement>(null);

    const handleMoreClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        if (moreButtonRef.current) {
            const rect = moreButtonRef.current.getBoundingClientRect();
            setMenuPosition({ x: rect.left, y: rect.bottom + 5 });
            setShowMoreMenu(!showMoreMenu);
        }
    };

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files.length > 0 && onFileUpload) {
            onFileUpload(e.target.files);
        }
    };

    return (
        <div className="action-toolbar">
            {/* Left Side: Primary Actions */}
            <div className="toolbar-left">
                <button className="btn-primary" onClick={onUpload}>
                    <span className="icon">↑</span> Upload
                </button>

                <button className="btn-secondary" onClick={onNewFolder}>
                    <span className="icon">+</span> New Folder
                </button>
                <button className="btn-secondary" onClick={onNewFile}>
                    <span className="icon">📄</span> New File
                </button>

            </div>

            {/* Right Side: View & File Actions */}
            <div className="toolbar-right">
                {selectedCount > 0 && (
                    <div className="action-group">
                        <button className="btn-icon-text" onClick={onDownload}>
                            <span className="icon">⬇️</span> Download
                        </button>
                        <button className="btn-icon-text">
                            <span className="icon">📑</span> Export Directory
                        </button>
                        <button className="btn-icon-text" onClick={onDelete}>
                            <span className="icon">🗑️</span> Delete
                        </button>
                        <button
                            className="btn-icon-text"
                            ref={moreButtonRef}
                            onClick={handleMoreClick}
                        >
                            <span className="icon">⋯</span>
                        </button>
                    </div>
                )}

                {showMoreMenu && (
                    <MoreActionsMenu
                        x={menuPosition.x}
                        y={menuPosition.y}
                        onClose={() => setShowMoreMenu(false)}
                        selectedCount={selectedCount}
                        onMove={onMove}
                        onCopy={onCopy}
                        onRename={onRename}
                        onDelete={onDelete}
                        onNewFolder={onNewFolder}
                        onDetails={onDetails}
                        onDownload={onDownload}
                    />
                )}

                <div className="divider"></div>

                <div className="view-toggle">
                    <button
                        className={`view-btn ${viewMode === 'grid' ? 'active' : ''}`}
                        onClick={() => onViewModeChange('grid')}
                        title="Grid View"
                    >
                        ⊞
                    </button>
                    <button
                        className={`view-btn ${viewMode === 'list' ? 'active' : ''}`}
                        onClick={() => onViewModeChange('list')}
                        title="List View"
                    >
                        ≡
                    </button>
                    <button className="view-btn" title="Refresh">
                        ↻
                    </button>
                </div>
            </div>
            <input
                type="file"
                ref={fileInputRef}
                style={{ display: 'none' }}
                multiple
                onChange={handleFileChange}
            />
        </div>
    );
};

export default ActionToolbar;
