import React from 'react';
import type { FileItem, SortOption } from '../../types';
import { Menu, MenuItem, MenuSeparator, SubMenu } from '../common/Menu';
import {
    IconOpen, IconDownload, IconCopy, IconCut,
    IconRename, IconMove, IconDetails, IconDelete,
    IconFolderPlus, IconUpload
} from '../common/Icons';
import { Colors } from '../../theme/colors';

interface ContextMenuProps {
    x: number;
    y: number;
    type: 'empty' | 'file';
    file: FileItem | null;
    sortBy: SortOption;
    onClose: () => void;
    onNewFolder: () => void;
    onUploadClick: () => void;
    onSort: (sort: SortOption) => void;
    onOpen: (file: FileItem) => void;
    onDownload: (file: FileItem) => void;
    onRename: (file: FileItem) => void;
    onDelete: (file: FileItem) => void;
    onDetails: (file: FileItem) => void;
}

const ContextMenu: React.FC<ContextMenuProps> = ({
    x,
    y,
    type,
    file,
    sortBy,
    onClose,
    onNewFolder,
    onUploadClick,
    onSort,
    onOpen,
    onDownload,
    onRename,
    onDelete,
    onDetails
}) => {
    if (type === 'empty') {
        return (
            <Menu x={x} y={y} onClose={onClose}>
                <MenuItem
                    label="New Folder"
                    icon={<IconFolderPlus color={Colors.folder} />}
                    onClick={() => { onNewFolder(); onClose(); }}
                />
                <MenuItem
                    label="Upload Files"
                    icon={<IconUpload color={Colors.upload} />}
                    onClick={() => { onUploadClick(); onClose(); }}
                />
                <MenuSeparator />
                <SubMenu label="Sort By" icon={<span style={{ fontSize: '1.2em' }}>⇅</span>}>
                    <MenuItem
                        label="Name"
                        icon={sortBy === 'name' ? <span>✓</span> : null}
                        onClick={() => { onSort('name'); onClose(); }}
                    />
                    <MenuItem
                        label="Size"
                        icon={sortBy === 'size' ? <span>✓</span> : null}
                        onClick={() => { onSort('size'); onClose(); }}
                    />
                    <MenuItem
                        label="Date Modified"
                        icon={sortBy === 'date' ? <span>✓</span> : null}
                        onClick={() => { onSort('date'); onClose(); }}
                    />
                </SubMenu>
            </Menu>
        );
    }

    if (!file) return null;

    return (
        <Menu x={x} y={y} onClose={onClose}>
            <MenuItem
                label="Open"
                icon={<IconOpen color={Colors.folder} />}
                onClick={() => { onOpen(file); onClose(); }}
            />
            <MenuItem
                label="Download"
                icon={<IconDownload color={Colors.download} />}
                onClick={() => { onDownload(file); onClose(); }}
            />

            <MenuSeparator />

            <MenuItem
                label="Copy"
                icon={<IconCopy color={Colors.copy} />}
                onClick={() => { /* TODO: Copy */ onClose(); }}
            />
            <MenuItem
                label="Cut"
                icon={<IconCut color={Colors.cut} />}
                onClick={() => { /* TODO: Cut */ onClose(); }}
            />
            <MenuItem
                label="Rename"
                icon={<IconRename color={Colors.rename} />}
                onClick={() => { onRename(file); onClose(); }}
            />
            <MenuItem
                label="Move to"
                icon={<IconMove color={Colors.move} />}
                onClick={() => { /* TODO: Move */ onClose(); }}
            />

            <MenuSeparator />

            <MenuItem
                label="Details"
                icon={<IconDetails color={Colors.details} />}
                onClick={() => { onDetails(file); onClose(); }}
            />

            <MenuSeparator />

            <MenuItem
                label="Delete"
                icon={<IconDelete color={Colors.delete} />}
                danger
                onClick={() => { onDelete(file); onClose(); }}
            />
        </Menu>
    );
};

export default ContextMenu;
