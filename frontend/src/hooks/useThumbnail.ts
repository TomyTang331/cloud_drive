import { useState, useEffect } from 'react';
import { fileService } from '../services/api';
import type { FileItem } from '../types';

export const useThumbnail = (file: FileItem) => {
    const [thumbnailUrl, setThumbnailUrl] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<Error | null>(null);

    useEffect(() => {
        let active = true;

        if (file.mime_type?.startsWith('image/')) {
            setLoading(true);
            fileService.downloadFile(file.id)
                .then((response: any) => {
                    if (active) {
                        const url = URL.createObjectURL(response.data);
                        setThumbnailUrl(url);
                        setError(null);
                    }
                })
                .catch((err: any) => {
                    if (active) {
                        console.error('Failed to load thumbnail:', err);
                        setError(err);
                        setThumbnailUrl(null);
                    }
                })
                .finally(() => {
                    if (active) {
                        setLoading(false);
                    }
                });
        } else {
            setThumbnailUrl(null);
            setLoading(false);
            setError(null);
        }

        return () => {
            active = false;
            if (thumbnailUrl) {
                URL.revokeObjectURL(thumbnailUrl);
            }
        };
    }, [file.id, file.mime_type]);

    return { thumbnailUrl, loading, error };
};
