import React from 'react';
import { Menu, MenuItem, MenuSeparator } from '../common/Menu';

interface MoreActionsMenuProps {
    x: number;
    y: number;
    onClose: () => void;
    selectedCount: number;
    onMove?: () => void;
    onCopy?: () => void;
    onRename?: () => void;
    onDelete: () => void;
    onNewFolder: () => void;
    onDetails?: () => void;
    onDownload: () => void;
}

const MoreActionsMenu: React.FC<MoreActionsMenuProps> = ({
    x,
    y,
    onClose,
    selectedCount,
    onMove,
    onCopy,
    onRename,
    onDelete,
    onNewFolder,
    onDetails,
    onDownload
}) => {
    return (
        <Menu x={x} y={y} onClose={onClose} className="menu-no-icons">
            <MenuItem label="Move to" onClick={() => { onMove?.(); onClose(); }} />
            <MenuItem label="Copy to" onClick={() => { onCopy?.(); onClose(); }} />
            <MenuItem
                label="Rename"
                onClick={() => { onRename?.(); onClose(); }}
                disabled={selectedCount !== 1}
            />
            <MenuItem
                label="Delete"
                onClick={() => { onDelete(); onClose(); }}
                danger
            />
            <MenuSeparator />
            <MenuItem
                label="New Folder"
                onClick={() => { onNewFolder(); onClose(); }}
            />
            <MenuItem
                label="Details"
                onClick={() => { onDetails?.(); onClose(); }}
            />
            <MenuSeparator />
            <MenuItem
                label="Download"
                onClick={() => { onDownload(); onClose(); }}
            />
        </Menu>
    );
};

export default MoreActionsMenu;
