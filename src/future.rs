// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.

use crate::{new_span, spawn_scope, Scope};
use std::task::Poll;

impl<T: Sized> FutureExt for T {}

pub trait FutureExt: Sized {
    #[inline]
    fn in_new_scope(self, event: &'static str) -> NewScope<Self> {
        NewScope {
            inner: self,
            event,
            scope: spawn_scope(event),
        }
    }

    #[inline]
    fn in_new_span(self, event: &'static str) -> NewSpan<Self> {
        NewSpan { inner: self, event }
    }
}

#[pin_project::pin_project]
pub struct NewScope<T> {
    #[pin]
    inner: T,
    event: &'static str,
    scope: Scope,
}

impl<T: std::future::Future> std::future::Future for NewScope<T> {
    type Output = T::Output;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.scope.start_scope();
        match this.inner.poll(cx) {
            r @ Poll::Pending => r,
            other => {
                this.scope.release();
                other
            }
        }
    }
}

impl<T: futures_01::Future> futures_01::Future for NewScope<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> futures_01::Poll<Self::Item, Self::Error> {
        let _guard = self.scope.start_scope();
        match self.inner.poll() {
            r @ Ok(futures_01::Async::NotReady) => r,
            other => {
                self.scope.release();
                other
            }
        }
    }
}

#[pin_project::pin_project]
pub struct NewSpan<T> {
    #[pin]
    inner: T,
    event: &'static str,
}

impl<T: std::future::Future> std::future::Future for NewSpan<T> {
    type Output = T::Output;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = new_span(this.event);
        this.inner.poll(cx)
    }
}

impl<T: futures_01::Future> futures_01::Future for NewSpan<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> futures_01::Poll<Self::Item, Self::Error> {
        let _guard = new_span(self.event);
        self.inner.poll()
    }
}
