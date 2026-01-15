import { useState, useMemo } from 'react';
import type { FileItem, SortOption, CategoryOption } from '../types';

export const useFileFilter = (files: FileItem[]) => {
    const [selectedCategory, setSelectedCategory] = useState<CategoryOption>('all');
    const [searchQuery, setSearchQuery] = useState('');
    const [sortBy, setSortBy] = useState<SortOption>('name');

    const processedFiles = useMemo(() => {
        let result = [...files];

        // Filter by Category
        if (selectedCategory !== 'all') {
            result = result.filter(file => {
                if (file.file_type === 'folder') return true;

                if (selectedCategory === 'recent') {
                    const date = new Date(file.created_at);
                    const now = new Date();
                    const diffTime = Math.abs(now.getTime() - date.getTime());
                    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
                    return diffDays <= 7;
                }

                if (!file.mime_type) return false;

                if (selectedCategory === 'images') return file.mime_type.startsWith('image/');
                if (selectedCategory === 'videos') return file.mime_type.startsWith('video/');
                if (selectedCategory === 'documents') return file.mime_type.includes('pdf') || file.mime_type.includes('document') || file.mime_type.includes('text') || file.mime_type.includes('msword') || file.mime_type.includes('sheet') || file.mime_type.includes('presentation');

                return true;
            });
        }

        // Filter by Search
        if (searchQuery) {
            result = result.filter(file => file.name.toLowerCase().includes(searchQuery.toLowerCase()));
        }

        // Sort
        return result.sort((a, b) => {
            // Folders always on top
            if (a.file_type !== b.file_type) {
                return a.file_type === 'folder' ? -1 : 1;
            }

            if (sortBy === 'name') {
                return a.name.localeCompare(b.name);
            } else if (sortBy === 'size') {
                return (b.size_bytes || 0) - (a.size_bytes || 0);
            } else {
                return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
            }
        });
    }, [files, selectedCategory, searchQuery, sortBy]);

    return {
        selectedCategory,
        setSelectedCategory,
        searchQuery,
        setSearchQuery,
        sortBy,
        setSortBy,
        processedFiles
    };
};
