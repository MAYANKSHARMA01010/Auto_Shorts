import sys
import json
import argparse
import numpy as np
import cv2
import librosa
import mediapipe as mp

def analyze_audio(audio_path, transcript_data):
    try:
        y, sr = librosa.load(audio_path, sr=16000)
    except Exception as e:
        print(f"Error loading audio: {e}")
        return

    for segment in transcript_data.get('segments', []):
        start = float(segment.get('start', 0.0))
        end = float(segment.get('end', 0.0))
        text = segment.get('text', '')
        
        start_sample = int(start * sr)
        end_sample = int(end * sr)
        
        segment_audio = y[start_sample:end_sample]
        
        if len(segment_audio) > 0:
            rms = np.sqrt(np.mean(segment_audio**2))
            # Convert to dB
            db = 20 * np.log10(rms + 1e-9)
            is_silence = db < -40.0
        else:
            db = -100.0
            is_silence = True
            
        duration = end - start
        if duration > 0:
            words = len(text.split())
            speaking_rate = words / duration
        else:
            speaking_rate = 0.0
            
        segment['audioMetadata'] = {
            'averageVolume': float(db),
            'speakingRate': float(speaking_rate),
            'isSilence': bool(is_silence)
        }

def analyze_video(video_path, transcript_data):
    cap = cv2.VideoCapture(video_path)
    if not cap.isOpened():
        print(f"Error opening video: {video_path}")
        return
        
    fps = cap.get(cv2.CAP_PROP_FPS)
    if fps <= 0: fps = 30
    
    # Target 2 FPS for lightweight processing
    frame_skip = int(fps / 2)
    if frame_skip < 1: frame_skip = 1
    
    mp_face_detection = mp.solutions.face_detection
    face_detection = mp_face_detection.FaceDetection(model_selection=0, min_detection_confidence=0.5)
    
    frame_metrics = []
    
    prev_gray = None
    frame_idx = 0
    current_time = 0.0
    
    while True:
        ret, frame = cap.read()
        if not ret:
            break
            
        if frame_idx % frame_skip == 0:
            current_time = frame_idx / fps
            gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
            # Resize for speed
            small_gray = cv2.resize(gray, (320, 180))
            
            # Motion & Scene Change
            motion = 0.0
            scene_change = False
            if prev_gray is not None:
                diff = cv2.absdiff(small_gray, prev_gray)
                motion = np.mean(diff)
                if motion > 30.0:  # Threshold for scene change
                    scene_change = True
            
            prev_gray = small_gray
            
            # Face Detection
            small_rgb = cv2.cvtColor(cv2.resize(frame, (320, 180)), cv2.COLOR_BGR2RGB)
            results = face_detection.process(small_rgb)
            face_present = results.detections is not None and len(results.detections) > 0
            
            frame_metrics.append({
                'time': current_time,
                'motion': motion,
                'scene_change': scene_change,
                'face_present': face_present
            })
            
        frame_idx += 1

    cap.release()
    
    if not frame_metrics:
        return
        
    # Aggregate into segments
    for segment in transcript_data.get('segments', []):
        start = float(segment.get('start', 0.0))
        end = float(segment.get('end', 0.0))
        
        # Find frames in this segment
        seg_frames = [f for f in frame_metrics if start <= f['time'] <= end]
        
        if not seg_frames:
            segment['visualMetadata'] = {
                'sceneChange': False,
                'motionLevel': "low",
                'facePresent': False
            }
            continue
            
        has_scene_change = any(f['scene_change'] for f in seg_frames)
        has_face = any(f['face_present'] for f in seg_frames)
        avg_motion = np.mean([f['motion'] for f in seg_frames])
        
        if avg_motion < 5.0:
            motion_level = "low"
        elif avg_motion < 15.0:
            motion_level = "medium"
        else:
            motion_level = "high"
            
        segment['visualMetadata'] = {
            'sceneChange': bool(has_scene_change),
            'motionLevel': motion_level,
            'facePresent': bool(has_face)
        }

def main():
    parser = argparse.ArgumentParser(description="Enrich transcript with multimodal signals.")
    parser.add_argument("--video", required=True, help="Path to video file")
    parser.add_argument("--audio", required=True, help="Path to audio file")
    parser.add_argument("--transcript", required=True, help="Path to input transcript JSON")
    parser.add_argument("--output", required=True, help="Path to output transcript JSON")
    
    args = parser.parse_args()
    
    with open(args.transcript, 'r', encoding='utf-8') as f:
        transcript_data = json.load(f)
        
    print("Analyzing audio...")
    analyze_audio(args.audio, transcript_data)
    
    print("Analyzing video...")
    analyze_video(args.video, transcript_data)
    
    with open(args.output, 'w', encoding='utf-8') as f:
        json.dump(transcript_data, f, indent=2, ensure_ascii=False)
        
    print(f"Enriched transcript written to {args.output}")

if __name__ == "__main__":
    main()
