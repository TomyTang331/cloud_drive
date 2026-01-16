import api from './client';
import type { ApiResponse, StorageInfo } from '../../types';

export const storageService = {
    getStorageInfo: () =>
        api.get<ApiResponse<StorageInfo>>('/api/storage/info'),
};
