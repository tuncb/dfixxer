unit Repro_Combined_RecoveryDrift_Generics;

interface

implementation

uses
  System.Generics.Collections;

procedure Touch(var AAccumulator: Integer; const APhase: Integer; const AStep: Integer);
begin
  Inc(AAccumulator, APhase + AStep);
end;

function GetStepsForPhase(const APhase: Integer): TArray<Integer>;
begin
  Result := TArray<Integer>.Create(APhase, APhase + 1);
end;

function LoadData(const APhases: TArray<Integer>; var AAccumulator: Integer): TArray<Integer>;
begin
  // Findings combined:
  // 1) block-local "const" is parsed as ERROR
  // 2) "for var ... in ..." is parsed as ERROR
  // 3) nested usage plus a multi-arg call can make recovery drift into following code
  const LData = TList<Integer>.Create();

  for var LPhase in APhases do
  begin
    const LLocal = LPhase;
    for var LStep in GetStepsForPhase(LLocal) do
    begin
      Touch(AAccumulator, LPhase, LStep);
      LData.Add(LStep);
    end;
  end;

  // In failing parses this tail statement can end up inside ERROR recovery.
  Result := LData.ToArray();
end;

function GenericAfterBoundary: TArray<Integer>;
begin
  // If the parser loses structure before here, generic context may be missing later.
  // Then formatter logic may treat '<' and '>' like operators instead of generic brackets.
  Result := nil;
end;

end.
