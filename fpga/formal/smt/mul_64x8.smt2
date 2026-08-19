; 64x8: __mul_noop vs bvmul, low 64 bits
(set-logic QF_BV)
(declare-fun a () (_ BitVec 64))
(declare-fun b () (_ BitVec 8))
(define-fun noop () (_ BitVec 64) (bvadd (bvadd (bvadd (bvadd (bvadd (bvadd (bvadd (ite (= ((_ extract 0 0) b) #b1) a (_ bv0 64)) (ite (= ((_ extract 1 1) b) #b1) (bvshl a (_ bv1 64)) (_ bv0 64))) (ite (= ((_ extract 2 2) b) #b1) (bvshl a (_ bv2 64)) (_ bv0 64))) (ite (= ((_ extract 3 3) b) #b1) (bvshl a (_ bv3 64)) (_ bv0 64))) (ite (= ((_ extract 4 4) b) #b1) (bvshl a (_ bv4 64)) (_ bv0 64))) (ite (= ((_ extract 5 5) b) #b1) (bvshl a (_ bv5 64)) (_ bv0 64))) (ite (= ((_ extract 6 6) b) #b1) (bvshl a (_ bv6 64)) (_ bv0 64))) (ite (= ((_ extract 7 7) b) #b1) (bvshl a (_ bv7 64)) (_ bv0 64))))
(define-fun golden () (_ BitVec 64) (bvmul a ((_ zero_extend 56) b)))
(assert (not (= noop golden)))
(check-sat)