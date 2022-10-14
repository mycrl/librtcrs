﻿#include <stdio.h>

#include "sys.h"
#include "convert.h"
#include "observer.h"

#include "api/peer_connection_interface.h"
#include "api/create_peerconnection_factory.h"
#include "api/audio_codecs/builtin_audio_decoder_factory.h"
#include "api/audio_codecs/builtin_audio_encoder_factory.h"
#include "api/video_codecs/builtin_video_decoder_factory.h"
#include "api/video_codecs/builtin_video_encoder_factory.h"

void rtc_run()
{
    rtc::Thread::Current()->Run();
}

RTCPeerConnection* create_rtc_peer_connection(
    RTCPeerConnectionConfigure* c_config,
    EventBus events)
{
    RTCPeerConnection* rtc = new RTCPeerConnection();
    rtc->pc_factory = webrtc::CreatePeerConnectionFactory(
        nullptr,
        nullptr,
        nullptr,
        nullptr,
        webrtc::CreateBuiltinAudioEncoderFactory(),
        webrtc::CreateBuiltinAudioDecoderFactory(),
        webrtc::CreateBuiltinVideoEncoderFactory(),
        webrtc::CreateBuiltinVideoDecoderFactory(),
        nullptr,
        nullptr);
    if (!rtc->pc_factory)
    {
        return NULL;
    }

    rtc::scoped_refptr<Observer> observer(
        new rtc::RefCountedObject<Observer>(events));
    observer->AddRef();

    rtc->pc = rtc->pc_factory->CreatePeerConnection(
        from_c(c_config),
        nullptr,
        nullptr,
        observer);
    if (!rtc->pc)
    {
        return NULL;
    }

    return rtc;
}

void rtc_close(RTCPeerConnection* peer)
{
    delete peer;
}

void rtc_add_ice_candidate(RTCPeerConnection* rtc, 
    RTCIceCandidate* icecandidate)
{
    rtc->pc->AddIceCandidate(from_c(icecandidate));
}

void rtc_create_answer(RTCPeerConnection* rtc, CreateDescCallback callback, 
    void* ctx)
{
    rtc->pc->CreateAnswer(
        DummyCreateDescObserver::Create(callback, ctx),
        webrtc::PeerConnectionInterface::RTCOfferAnswerOptions());
}

void rtc_create_offer(RTCPeerConnection* rtc, CreateDescCallback callback, 
    void* ctx)
{
    rtc->pc->CreateOffer(
        DummyCreateDescObserver::Create(callback, ctx),
        webrtc::PeerConnectionInterface::RTCOfferAnswerOptions());
}

void rtc_set_local_description(RTCPeerConnection* rtc, 
    RTCSessionDescription* c_desc, 
    SetDescCallback callback, 
    void* ctx)
{
    rtc->pc->SetLocalDescription(
        DummySetDescObserver::Create(callback, ctx),
        from_c(c_desc).release());
}

void rtc_set_remote_description(RTCPeerConnection* rtc,
    RTCSessionDescription* c_desc,
    SetDescCallback callback,
    void* ctx)
{
    rtc->pc->SetRemoteDescription(
        DummySetDescObserver::Create(callback, ctx),
        from_c(c_desc).release());
}

void rtc_add_track(RTCPeerConnection* rtc,
    MediaStreamTrack* track)
{
    // rtc->pc_factory->CreateVideoTrack();
}