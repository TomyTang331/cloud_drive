import React, { useEffect, useState, useRef } from 'react';
import videojs from 'video.js';
import 'video.js/dist/video-js.css';
import { fileService } from '../../services/api';
import type { FileItem } from '../../types';
import { useToast } from '../../hooks/useToast';
import './VideoPlayerModal.less';

interface VideoPlayerModalProps {
    isOpen: boolean;
    onClose: () => void;
    file: FileItem | null;
    onDelete?: (file: FileItem) => void;
}

const VideoPlayerModal: React.FC<VideoPlayerModalProps> = ({
    isOpen,
    onClose,
    file,
    onDelete
}) => {
    const toast = useToast();
    const [videoUrl, setVideoUrl] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [showDetails, setShowDetails] = useState(false);
    const videoNode = useRef<HTMLVideoElement>(null);
    const player = useRef<any>(null);

    // Fetch video URL
    useEffect(() => {
        if (!isOpen || !file) {
            setVideoUrl(null);
            setError(null);
            setShowDetails(false);
            return;
        }

        const fetchVideo = async () => {
            setLoading(true);
            setError(null);
            try {
                const response = await fileService.downloadFile(file.id);
                const url = URL.createObjectURL(response.data);
                setVideoUrl(url);
            } catch (err: any) {
                console.error('Failed to load video:', err);
                setError(`Failed to load video: ${err.message || 'Unknown error'}`);
            } finally {
                setLoading(false);
            }
        };

        fetchVideo();

        return () => {
            if (videoUrl) {
                URL.revokeObjectURL(videoUrl);
            }
        };
    }, [isOpen, file]);

    // Initialize Video.js
    useEffect(() => {
        if (!videoNode.current || !videoUrl) return;

        const videoJsOptions = {
            autoplay: true,
            controls: true,
            responsive: true,
            fluid: true,
            sources: [{
                src: videoUrl,
                type: file?.mime_type || 'video/mp4'
            }],
            controlBar: {
                children: [
                    'playToggle',
                    'currentTimeDisplay',
                    'progressControl',
                    'durationDisplay',
                    'volumePanel',
                    'fullscreenToggle',
                ]
            }
        };

        // Initialize player
        player.current = videojs(videoNode.current, videoJsOptions, () => {

        });

        // Cleanup
        return () => {
            if (player.current) {
                player.current.dispose();
                player.current = null;
            }
        };
    }, [videoUrl, file, isOpen]);

    // Cleanup on unmount or close
    useEffect(() => {
        return () => {
            if (player.current) {
                player.current.dispose();
                player.current = null;
            }
        };
    }, []);

    // Keyboard navigation
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            } else if (e.key === ' ' && player.current) {
                e.preventDefault();
                if (player.current.paused()) {
                    player.current.play();
                } else {
                    player.current.pause();
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onClose]);

    const handleDownload = async () => {
        if (!file) return;
        try {
            const response = await fileService.downloadFile(file.id);
            const url = URL.createObjectURL(response.data);
            const a = document.createElement('a');
            a.href = url;
            a.download = file.name;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } catch (err) {
            console.error('Download failed:', err);
            toast.error('Failed to download file');
        }
    };

    const handleDelete = () => {
        if (file && onDelete) {
            onDelete(file);
            onClose();
        }
    };

    if (!isOpen || !file) return null;

    return (
        <div className="video-player-overlay" onClick={onClose}>
            <div className={`video-player-container ${showDetails ? 'with-details' : ''}`} onClick={(e) => e.stopPropagation()}>

                {/* Header Bar */}
                <div className="player-header">
                    <div className="player-title">
                        <h3>{file.name}</h3>
                        <span className="player-meta">
                            {new Date(file.created_at).toLocaleString()}
                        </span>
                    </div>
                    <div className="header-actions">
                        <button
                            className={`action-icon-btn ${showDetails ? 'active' : ''}`}
                            onClick={() => setShowDetails(!showDetails)}
                            title="Info"
                        >
                            ℹ️
                        </button>
                        <button className="player-close-btn" onClick={onClose} title="Close">
                            ✕
                        </button>
                    </div>
                </div>

                {/* Main Content: Video + Sidebar */}
                <div className="player-body">
                    <div className="video-area">
                        {loading ? (
                            <div className="player-loading">
                                <div className="spinner"></div>
                                <p>Loading video...</p>
                            </div>
                        ) : error ? (
                            <div className="player-error">
                                <div className="error-icon">⚠️</div>
                                <p>{error}</p>
                            </div>
                        ) : (
                            <div data-vjs-player style={{ width: '100%', maxWidth: '100%', height: '100%' }}>
                                <video
                                    ref={videoNode}
                                    className="video-js vjs-big-play-centered"
                                />
                            </div>
                        )}
                    </div>

                    {/* Details Sidebar */}
                    {showDetails && (
                        <div className="details-sidebar">
                            <div className="sidebar-content">
                                <div className="detail-group">
                                    <label>FILE NAME</label>
                                    <p>{file.name}</p>
                                </div>
                                <div className="detail-group">
                                    <label>TYPE</label>
                                    <p>{file.mime_type || 'video/mp4'}</p>
                                </div>
                                <div className="detail-group">
                                    <label>SIZE</label>
                                    <p>{file.size_bytes ? (file.size_bytes / 1024 / 1024).toFixed(2) + ' MB' : 'Unknown'}</p>
                                </div>
                                <div className="detail-group">
                                    <label>LOCATION</label>
                                    <p>{file.path}</p>
                                </div>
                                <div className="detail-group">
                                    <label>CREATED</label>
                                    <p>{new Date(file.created_at).toLocaleString()}</p>
                                </div>
                                <div className="detail-group">
                                    <label>MODIFIED</label>
                                    <p>{new Date(file.updated_at).toLocaleString()}</p>
                                </div>
                            </div>
                        </div>
                    )}
                </div>

                {/* Footer Bar */}
                <div className="player-footer">
                    <div className="footer-actions">
                        <button className="footer-btn" onClick={handleDownload}>
                            <span className="btn-icon">⬇</span>
                            Download
                        </button>
                        {file.can_delete && (
                            <button className="footer-btn delete" onClick={handleDelete}>
                                <span className="btn-icon">🗑️</span>
                                Delete
                            </button>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};

export default VideoPlayerModal;
