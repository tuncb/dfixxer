unit ReproLtLength;
interface
implementation

procedure P(const X: Integer; const A: TArray<Integer>);
begin
  Assert(X < Length(A), 'msg');
end;

end.
