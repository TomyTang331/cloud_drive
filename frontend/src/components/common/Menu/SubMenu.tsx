import React, { useState } from 'react';
import { MenuItem } from './MenuItem';
import type { MenuItemProps } from './MenuItem';
import './Menu.less';

interface SubMenuProps extends Omit<MenuItemProps, 'onClick'> {
    children: React.ReactNode;
}

export const SubMenu: React.FC<SubMenuProps> = ({ children, ...menuItemProps }) => {
    const [isOpen, setIsOpen] = useState(false);

    return (
        <div
            className="submenu-wrapper"
            onMouseEnter={() => setIsOpen(true)}
            onMouseLeave={() => setIsOpen(false)}
            style={{ position: 'relative', width: '100%' }}
        >
            <MenuItem
                {...menuItemProps}
                className={isOpen ? 'active' : ''}
            >
                <span className="menu-item-arrow">▶</span>
            </MenuItem>

            {isOpen && (
                <div className="submenu-container menu-container">
                    {children}
                </div>
            )}
        </div>
    );
};
